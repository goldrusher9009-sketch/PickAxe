//! Studying a public web page from the outside, safely enough to run on a shared server.
//!
//! `core_page_study` fetches one http(s) page and reports what it is made of: its address after
//! redirects, response headers, the scripts, styles, forms and links in its HTML, and signs of the
//! stack behind it. It never runs the page's scripts and never opens a browser, so nothing a site
//! sends can act on the machine. Every connection (the first and each redirect) goes through a
//! resolver that refuses private, loopback, link-local and other non-public addresses, so the tool
//! can't be pointed at the host's own network.

use super::artifact_tools::env;
use super::native_common::params_json;
use crate::tool::{Ctx, Effects, Requires, ToolOutput, ToolSpec, Toolset, parse_params, schema_of};
use pickaxe_evidence::{Authority, Confidence, Coverage, CoverageStatus, ErrorCode, EvidenceBuilder, Limitation, Location, Subject, ToolError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::Duration;
use ureq::ResponseExt;
use ureq::config::Config;
use ureq::http::Uri;
use ureq::unversioned::resolver::{DefaultResolver, ResolvedSocketAddrs, Resolver};
use ureq::unversioned::transport::{DefaultConnector, NextTimeout};

/// The tool's id.
pub const PAGE_STUDY_TOOL: &str = "core_page_study";

pub fn specs() -> Vec<ToolSpec> {
    vec![ToolSpec {
        id: PAGE_STUDY_TOOL,
        title: "Study a web page",
        summary: "Fetch one public web page (http or https) and report how it is built: its final address after redirects, response headers, title and description, the scripts, styles, forms and links in its HTML, and signs of the framework or server behind it. Nothing on the page is run and no browser is opened. Refuses addresses on private or internal networks. Reads at most 2 MiB.",
        beginner: "Look at how a public web page is built, without running anything on it.",
        toolset: Toolset::Core,
        effects: Effects::EVIDENCE.union(Effects::ACCESSES_NETWORK),
        requires: Requires::Nothing,
        input_schema: schema_of::<PageStudyParams>,
        run: core_page_study,
    }]
}

/// Most of a page's body that is read.
const MAX_BODY: u64 = 2 << 20;
/// Most redirects followed.
const MAX_REDIRECTS: u32 = 5;
/// The whole fetch, DNS to last byte.
const TOTAL_TIME: Duration = Duration::from_secs(12);
/// Most items listed per kind.
const MAX_LIST: usize = 60;

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PageStudyParams {
    /// The page's address, `https://example.com/` (a bare host name is read as https).
    pub url: String,
}

/// True for an address on the public internet. Anything private, internal, reserved or special-use
/// is refused, including IPv4 hidden inside an IPv6 address.
pub fn is_public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => public_v4(v4),
        IpAddr::V6(v6) => public_v6(v6),
    }
}

fn public_v4(ip: Ipv4Addr) -> bool {
    let [a, b, c, _] = ip.octets();
    !(ip.is_unspecified()
        || ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_broadcast()
        || ip.is_multicast()
        || ip.is_documentation()
        || a == 0
        || (a == 100 && (64..=127).contains(&b))
        || (a == 192 && b == 0 && c == 0)
        || (a == 198 && (b == 18 || b == 19))
        || a >= 240)
}

fn public_v6(ip: Ipv6Addr) -> bool {
    if let Some(v4) = ip.to_ipv4_mapped() {
        return public_v4(v4);
    }
    let s = ip.segments();
    // 64:ff9b::/96 carries an IPv4 address in its last 32 bits.
    if s[0] == 0x64 && s[1] == 0xff9b && s[2..6].iter().all(|x| *x == 0) {
        let o = ip.octets();
        return public_v4(Ipv4Addr::new(o[12], o[13], o[14], o[15]));
    }
    !(ip.is_unspecified()
        || ip.is_loopback()
        || ip.is_multicast()
        || (s[0] & 0xfe00) == 0xfc00 // unique local fc00::/7
        || (s[0] & 0xffc0) == 0xfe80 // link-local fe80::/10
        || (s[0] == 0x2001 && s[1] == 0x0db8) // documentation
        || s[0] == 0x0100 && s[1..4].iter().all(|x| *x == 0)) // discard-only
}

/// Looks names up the usual way, then keeps only public addresses. Used for every connection, so a
/// redirect or a name that resolves to an internal address is refused the same way.
#[derive(Debug, Default)]
struct PublicOnly(DefaultResolver);

impl Resolver for PublicOnly {
    fn resolve(&self, uri: &Uri, config: &Config, timeout: NextTimeout) -> Result<ResolvedSocketAddrs, ureq::Error> {
        let all = self.0.resolve(uri, config, timeout)?;
        let mut kept = self.empty();
        for addr in all.iter().filter(|a: &&SocketAddr| is_public_ip(a.ip())) {
            kept.push(*addr);
        }
        if kept.is_empty() {
            return Err(ureq::Error::Other("that address is not on the public internet".into()));
        }
        Ok(kept)
    }
}

fn agent() -> ureq::Agent {
    let config = Config::builder()
        .http_status_as_error(false)
        .user_agent("Pickaxe-page-study")
        .max_redirects(MAX_REDIRECTS)
        .timeout_global(Some(TOTAL_TIME))
        .https_only(false)
        .build();
    ureq::Agent::with_parts(config, DefaultConnector::default(), PublicOnly::default())
}

/// The address to fetch, made explicit and checked before any connection.
fn parse_target(raw: &str) -> Result<Uri, ToolError> {
    let raw = raw.trim();
    if raw.is_empty() || raw.len() > 2048 {
        return Err(ToolError::invalid("give a web address of at most 2048 characters"));
    }
    let with_scheme = if raw.contains("://") { raw.to_string() } else { format!("https://{raw}") };
    let uri: Uri = with_scheme.parse().map_err(|_| ToolError::invalid(format!("{raw} isn't a web address")))?;
    match uri.scheme_str() {
        Some("http" | "https") => {}
        _ => return Err(ToolError::invalid("only http and https addresses can be studied")),
    }
    let Some(host) = uri.host() else { return Err(ToolError::invalid("that address has no host name")) };
    if uri.authority().is_some_and(|a| a.as_str().contains('@')) {
        return Err(ToolError::invalid("addresses with a user name or password aren't studied"));
    }
    if let Some(port) = uri.port_u16()
        && port != 80
        && port != 443
    {
        return Err(ToolError::new(ErrorCode::NoPermission, "only the standard web ports (80 and 443) can be studied"));
    }
    let literal = host.trim_matches(['[', ']']).parse::<IpAddr>();
    if let Ok(ip) = literal
        && !is_public_ip(ip)
    {
        return Err(ToolError::new(ErrorCode::NoPermission, "that address is on a private or internal network"));
    }
    if host.eq_ignore_ascii_case("localhost") || host.to_ascii_lowercase().ends_with(".localhost") || host.to_ascii_lowercase().ends_with(".internal") {
        return Err(ToolError::new(ErrorCode::NoPermission, "that address is on a private or internal network"));
    }
    Ok(uri)
}

/// What a page's HTML is made of, found without running any of it.
#[derive(Debug, Default, Serialize)]
struct PageFacts {
    title: Option<String>,
    meta: Vec<(String, String)>,
    scripts: Vec<String>,
    inline_scripts: usize,
    styles: Vec<String>,
    forms: Vec<Value>,
    links: usize,
    images: usize,
    json_ld: usize,
}

fn attr(tag: &str, name: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let mut from = 0;
    while let Some(i) = lower[from..].find(name) {
        let at = from + i;
        let before_ok = at == 0 || lower.as_bytes()[at - 1].is_ascii_whitespace();
        let rest = &tag[at + name.len()..];
        let trimmed = rest.trim_start();
        if before_ok && let Some(value) = trimmed.strip_prefix('=') {
            let value = value.trim_start();
            return Some(match value.chars().next() {
                Some(q @ ('"' | '\'')) => value[1..].split(q).next().unwrap_or("").to_string(),
                _ => value.split(|c: char| c.is_whitespace() || c == '>').next().unwrap_or("").to_string(),
            });
        }
        from = at + name.len();
    }
    None
}

fn clip(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

fn read_html(html: &str) -> PageFacts {
    let mut f = PageFacts::default();
    let lower = html.to_ascii_lowercase();
    if let (Some(a), Some(b)) = (lower.find("<title"), lower.find("</title>"))
        && let Some(gt) = lower[a..b].find('>')
    {
        f.title = Some(clip(html[a + gt + 1..b].trim(), 200));
    }
    let mut form: Option<(String, String, Vec<String>)> = None;
    let mut pos = 0;
    while let Some(i) = html[pos..].find('<') {
        let start = pos + i;
        let Some(end) = html[start..].find('>') else { break };
        let tag = &html[start + 1..start + end];
        pos = start + end + 1;
        let name = tag.split(|c: char| c.is_whitespace() || c == '/').next().unwrap_or("").to_ascii_lowercase();
        match name.as_str() {
            "meta" => {
                let key = attr(tag, "name").or_else(|| attr(tag, "property")).or_else(|| attr(tag, "http-equiv"));
                if let (Some(k), Some(v)) = (key, attr(tag, "content"))
                    && f.meta.len() < MAX_LIST
                {
                    f.meta.push((clip(&k, 60), clip(&v, 300)));
                }
            }
            "script" => match attr(tag, "src") {
                Some(src) if f.scripts.len() < MAX_LIST => f.scripts.push(clip(&src, 300)),
                Some(_) => {}
                None => {
                    f.inline_scripts += 1;
                    if attr(tag, "type").is_some_and(|t| t.eq_ignore_ascii_case("application/ld+json")) {
                        f.json_ld += 1;
                    }
                }
            },
            "link" => {
                if attr(tag, "rel").is_some_and(|r| r.to_ascii_lowercase().contains("stylesheet"))
                    && let Some(h) = attr(tag, "href")
                    && f.styles.len() < MAX_LIST
                {
                    f.styles.push(clip(&h, 300));
                }
            }
            "a" => f.links += 1,
            "img" => f.images += 1,
            "form" => form = Some((attr(tag, "action").unwrap_or_default(), attr(tag, "method").unwrap_or_else(|| "get".into()), Vec::new())),
            "input" | "select" | "textarea" => {
                if let Some((_, _, fields)) = form.as_mut()
                    && fields.len() < MAX_LIST
                {
                    let kind = attr(tag, "type").unwrap_or_else(|| name.clone());
                    fields.push(format!("{}:{}", attr(tag, "name").unwrap_or_default(), kind));
                }
            }
            "/form" => {
                if let Some((action, method, fields)) = form.take()
                    && f.forms.len() < MAX_LIST
                {
                    f.forms.push(json!({"action": clip(&action, 300), "method": method.to_ascii_lowercase(), "fields": fields}));
                }
            }
            _ => {}
        }
    }
    if let Some((action, method, fields)) = form.take() {
        f.forms.push(json!({"action": clip(&action, 300), "method": method.to_ascii_lowercase(), "fields": fields}));
    }
    f
}

/// Signs of what a site is built with, from its headers and HTML. These are hints, not proof.
fn stack_hints(headers: &[(String, String)], html: &str, facts: &PageFacts) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut add = |s: &str| {
        if !out.iter().any(|x| x == s) {
            out.push(s.to_string());
        }
    };
    for (k, v) in headers {
        match k.as_str() {
            "server" => add(&format!("server: {}", clip(v, 80))),
            "x-powered-by" => add(&format!("powered by: {}", clip(v, 80))),
            "x-vercel-id" => add("hosted on Vercel"),
            "x-nf-request-id" => add("hosted on Netlify"),
            "cf-ray" => add("behind Cloudflare"),
            "x-amz-cf-id" => add("behind Amazon CloudFront"),
            "x-shopify-stage" | "x-shopid" => add("Shopify"),
            _ => {}
        }
    }
    let l = html.to_ascii_lowercase();
    let scripts = facts.scripts.join(" ").to_ascii_lowercase();
    let checks: [(&str, &str); 14] = [
        ("__next_data__", "Next.js"),
        ("/_next/", "Next.js"),
        ("__nuxt", "Nuxt"),
        ("ng-version", "Angular"),
        ("data-reactroot", "React"),
        ("/wp-content/", "WordPress"),
        ("cdn.shopify.com", "Shopify"),
        ("wixstatic.com", "Wix"),
        ("squarespace", "Squarespace"),
        ("webflow", "Webflow"),
        ("gatsby", "Gatsby"),
        ("__sveltekit", "SvelteKit"),
        ("data-v-app", "Vue"),
        ("astro-island", "Astro"),
    ];
    for (needle, label) in checks {
        if l.contains(needle) || scripts.contains(needle) {
            add(label);
        }
    }
    for (k, v) in &facts.meta {
        if k.eq_ignore_ascii_case("generator") {
            add(&format!("generator: {}", clip(v, 80)));
        }
    }
    out
}

fn core_page_study(ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: PageStudyParams = parse_params(params)?;
    let uri = parse_target(&p.url)?;
    let shown = uri.to_string();
    ctx.report(None, format!("Fetching {shown}"));
    ctx.cancel.check()?;

    let mut response = agent().get(uri).call().map_err(|e| {
        let text = e.to_string();
        if text.contains("public internet") {
            ToolError::new(ErrorCode::NoPermission, "that address is on a private or internal network")
        } else if matches!(e, ureq::Error::Timeout(_)) {
            ToolError::new(ErrorCode::TimedOut, format!("{shown} didn't answer within {} seconds.", TOTAL_TIME.as_secs()))
        } else {
            ToolError::new(ErrorCode::FileError, format!("Couldn't fetch {shown}: {text}"))
        }
    })?;
    let status = response.status().as_u16();
    let final_url = response.get_uri().to_string();
    let headers: Vec<(String, String)> =
        response.headers().iter().take(60).map(|(k, v)| (k.as_str().to_ascii_lowercase(), clip(v.to_str().unwrap_or("(not text)"), 300))).collect();
    let content_type = headers.iter().find(|(k, _)| k == "content-type").map(|(_, v)| v.clone()).unwrap_or_default();
    let mut body = Vec::new();
    response
        .body_mut()
        .as_reader()
        .take(MAX_BODY + 1)
        .read_to_end(&mut body)
        .map_err(|e| ToolError::new(ErrorCode::FileError, format!("Couldn't read {shown}: {e}")))?;
    let cut = body.len() as u64 > MAX_BODY;
    body.truncate(MAX_BODY as usize);
    let text = String::from_utf8_lossy(&body);
    let is_html = content_type.to_ascii_lowercase().contains("html") || text.trim_start().to_ascii_lowercase().starts_with("<!doctype html");
    let facts = if is_html { read_html(&text) } else { PageFacts::default() };
    let hints = if is_html { stack_hints(&headers, &text, &facts) } else { stack_hints(&headers, "", &facts) };

    let mut limits = vec![Limitation::new(
        "static_view",
        "The page was fetched and read as text. Its scripts did not run, so anything a script builds, loads later or hides behind a sign-in is not here.",
    )];
    if cut {
        limits.push(Limitation::new("body_cut", "The page is larger than 2 MiB; only the first 2 MiB was read."));
    }
    if !is_html {
        limits.push(Limitation::new("not_html", format!("The answer is {content_type:?}, not a web page, so only its headers are reported.")));
    }
    let summary = match &facts.title {
        Some(t) => format!(
            "{final_url}: \"{t}\" (HTTP {status}, {} script(s), {} form(s), {} link(s)).",
            facts.scripts.len() + facts.inline_scripts,
            facts.forms.len(),
            facts.links
        ),
        None => format!("{final_url}: HTTP {status}, {} bytes read.", body.len()),
    };
    let mut coverage = Coverage::complete(body.len() as u64);
    if cut {
        coverage.status = CoverageStatus::Partial;
    }
    let ev = EvidenceBuilder::new(PAGE_STUDY_TOOL, Subject { kind: "website".into(), sha256: None, name: Some(shown.clone()) }, env(ctx))
        .params(params_json(&p))
        .result(json!({
            "url": final_url,
            "requested": shown,
            "status": status,
            "content_type": content_type,
            "bytes_read": body.len(),
            "headers": headers,
            "title": facts.title,
            "meta": facts.meta,
            "scripts": facts.scripts,
            "inline_scripts": facts.inline_scripts,
            "styles": facts.styles,
            "forms": facts.forms,
            "link_count": facts.links,
            "image_count": facts.images,
            "json_ld_blocks": facts.json_ld,
            "stack_hints": hints,
        }))
        .summary(summary)
        .confidence(Confidence::Observed)
        .authority(Authority::OurRun)
        .limitations(limits)
        .coverage(coverage)
        .location(Location::new("url", final_url))
        .build()?;
    Ok(ToolOutput::evidence(ev))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal_addresses_are_refused_and_public_ones_are_not() {
        for bad in [
            "127.0.0.1",
            "10.1.2.3",
            "192.168.0.9",
            "172.16.5.5",
            "169.254.169.254",
            "100.64.0.1",
            "0.0.0.0",
            "::1",
            "fe80::1",
            "fc00::1",
            "::ffff:10.0.0.1",
            "64:ff9b::7f00:1",
        ] {
            assert!(!is_public_ip(bad.parse().unwrap()), "{bad} should be refused");
        }
        for good in ["8.8.8.8", "93.184.216.34", "2606:4700:4700::1111"] {
            assert!(is_public_ip(good.parse().unwrap()), "{good} should be allowed");
        }
    }

    #[test]
    fn addresses_that_reach_the_host_are_refused_before_any_connection() {
        for bad in [
            "http://localhost/",
            "https://127.0.0.1/",
            "http://169.254.169.254/latest/meta-data/",
            "https://example.com:8080/",
            "ftp://example.com/",
            "https://user:pw@example.com/",
            "http://[::1]/",
            "http://service.internal/",
        ] {
            assert!(parse_target(bad).is_err(), "{bad} should be refused");
        }
        assert!(parse_target("example.com").is_ok(), "a bare host is read as https");
        assert!(parse_target("https://example.com/a?b=1").is_ok());
    }

    #[test]
    fn a_page_is_read_without_running_it() {
        let html = r#"<!doctype html><html><head><title>Shop</title><meta name="generator" content="WordPress 6">
            <link rel="stylesheet" href="/wp-content/a.css"><script src="/app.js"></script><script>var x=1</script></head>
            <body><a href="/a">a</a><a href="/b">b</a><form action="/login" method="POST"><input name="email" type="email"><input name="pw" type="password"></form></body></html>"#;
        let f = read_html(html);
        assert_eq!(f.title.as_deref(), Some("Shop"));
        assert_eq!(f.scripts, ["/app.js"]);
        assert_eq!(f.inline_scripts, 1);
        assert_eq!(f.links, 2);
        assert_eq!(f.forms.len(), 1);
        assert_eq!(f.forms[0]["method"], "post");
        assert!(f.forms[0]["fields"].as_array().unwrap().iter().any(|x| x == "pw:password"));
        let hints = stack_hints(&[("server".into(), "nginx".into())], html, &f);
        assert!(hints.iter().any(|h| h == "WordPress") && hints.iter().any(|h| h.contains("nginx")));
    }
}
