use std::sync::LazyLock;

use include_dir::{Dir, include_dir};
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Meta, Title};
use leptos_router::{
    components::{A, Route, Router, Routes},
    path,
};
use serde::Deserialize;

mod config {
    include!(concat!(env!("OUT_DIR"), "/config_generated.rs"));
}

#[derive(Clone, Copy, Debug)]
pub struct Project {
    pub name: &'static str,
    pub description: &'static str,
    pub language: &'static str,
    pub owner: &'static str,
    pub repo: &'static str,
}

impl Project {
    #[must_use]
    pub fn url(&self) -> String {
        format!("https://github.com/{}/{}", self.owner, self.repo)
    }
}

#[derive(Clone, Debug)]
pub struct BlogPost {
    pub slug: String,
    pub title: String,
    pub date: String,
    pub description: String,
    pub content: String,
}

#[derive(Clone, Debug, Deserialize)]
struct GitHubRepo {
    stargazers_count: u32,
}

static POSTS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/posts");

static YEAR_ROMAN: LazyLock<String> = LazyLock::new(|| to_roman(config::YEAR));

static BLOG_POSTS: LazyLock<Vec<BlogPost>> = LazyLock::new(|| {
    let mut posts = Vec::new();

    for file in POSTS_DIR.files() {
        if let Some(ext) = file.path().extension()
            && ext == "md"
            && let Some(content) = file.contents_utf8()
        {
            let slug = file
                .path()
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            let post = parse_post(&slug, content);
            posts.push(post);
        }
    }

    posts.sort_by(|a, b| b.date.cmp(&a.date));
    posts
});

fn to_roman(mut num: u32) -> String {
    let numerals = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];

    let mut result = String::new();
    for (value, symbol) in numerals {
        while num >= value {
            result.push_str(symbol);
            num -= value;
        }
    }
    result
}

fn parse_post(slug: &str, content: &str) -> BlogPost {
    let mut title = String::from("Untitled");
    let mut date = String::from("Unknown");
    let mut description = None;
    let mut body = content.to_string();

    if content.starts_with("---") {
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() >= 3 {
            let frontmatter = parts[1];
            body = parts[2].trim().to_string();

            for line in frontmatter.lines() {
                let line = line.trim();
                if let Some(t) = line.strip_prefix("title:") {
                    title = t.trim().trim_matches('"').to_string();
                } else if let Some(d) = line.strip_prefix("date:") {
                    date = d.trim().trim_matches('"').to_string();
                } else if let Some(desc) = line.strip_prefix("description:") {
                    description = Some(desc.trim().trim_matches('"').to_string());
                }
            }
        }
    }

    let description = description.unwrap_or_else(|| get_excerpt(&body));

    BlogPost {
        slug: slug.to_string(),
        title,
        date,
        description,
        content: body,
    }
}

fn get_excerpt(content: &str) -> String {
    let plain: String = content
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .take(2)
        .collect::<Vec<_>>()
        .join(" ");

    if plain.len() > 200 {
        format!("{}...", &plain[..200])
    } else {
        plain
    }
}

fn render_markdown(content: &str) -> String {
    use pulldown_cmark::{Options, Parser, html};

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(content, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
}

async fn fetch_github_stars(owner: &str, repo: &str) -> Option<u32> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}");

    let response = gloo_net::http::Request::get(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", "afkaradotdev")
        .send()
        .await
        .ok()?;

    if !response.ok() {
        return None;
    }

    let repo_data: GitHubRepo = response.json().await.ok()?;
    Some(repo_data.stargazers_count)
}

#[component]
#[allow(clippy::must_use_candidate)]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Router>
            <Routes fallback=|| view! { <NotFound /> }>
                <Route path=path!("/") view=HomePage />
                <Route path=path!("/blog") view=BlogPage />
                <Route path=path!("/blog/:slug") view=BlogPostPage />
            </Routes>
        </Router>
    }
}

#[component]
fn Header() -> impl IntoView {
    let year = YEAR_ROMAN.as_str();
    view! {
        <header class="masthead">
            <a href="/" class="masthead-link">
                <h1 class="masthead-title">"The Afkara Gazette"</h1>
            </a>
            <p class="masthead-subtitle">"Purveyor of Fine Computery Activities"</p>
            <div class="date-line">
                <span>"Est. " {year}</span>
                <span>"Volume I, Issue I"</span>
                <span>"Price: $0.00"</span>
            </div>
        </header>

        <nav>
            <a href="/#about">"About"</a>
            <a href="/#projects">"Projects"</a>
            <A href="/blog">"Blog"</A>
            <a href="/#contact">"Contact"</a>
        </nav>
    }
}

#[component]
fn Footer() -> impl IntoView {
    let year = YEAR_ROMAN.as_str();
    view! {
        <p class="ornament">"— ✿ —"</p>
        <footer class="footer">
            <p>"© " {year} " The Afkara Gazette — All Rights Reserved"</p>
        </footer>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let projects = &config::PROJECTS;
    let half = projects.len().div_ceil(2);

    view! {
        <div class="page">
            <Header />

            <h2 class="headline">"Developer & Creator Extraordinaire"</h2>
            <p class="subheadline">"Creating Bits and Bytes + Occasional Digital Noises"</p>

            <div class="divider">
                <span class="divider-ornament">"§"</span>
            </div>

            <section id="about">
                <h3 class="section-header">"About the Publisher"</h3>
                <p class="drop-cap">
                    "Greetings, dear reader! I am AfkaraLP, a maker of music and software in equal measure. My primary pursuit lies in the creation of Future Bounce—a genre that brings me great joy (They forced me to say this). By day, I pursue the scholarly art of Data Science as a Bachelor's student, seeking to unravel the mysteries hidden within numbers and patterns."
                </p>
                <p class="article">
                    "My weapon of choice in the realm of programming is Rust, a language of elegance and safety. I am a devoted user of NixOS (by the way) and harbour a deep appreciation for the functional programming paradigm. Beyond the digital realm, one might find me engaged in sporting activities or experimenting in the culinary arts."
                </p>
            </section>

            <div class="divider">
                <span class="divider-ornament">"❧"</span>
            </div>

            <section id="projects">
                <h3 class="section-header">"Notable Works & Endeavours"</h3>

                <div class="columns">
                    <div class="column">
                        {projects
                            .iter()
                            .take(half)
                            .map(|p| view! { <ProjectCard project=*p /> })
                            .collect::<Vec<_>>()}
                    </div>
                    <div class="column">
                        {projects
                            .iter()
                            .skip(half)
                            .map(|p| view! { <ProjectCard project=*p /> })
                            .collect::<Vec<_>>()}
                    </div>
                </div>
            </section>

            <div class="divider">
                <span class="divider-ornament">"✦"</span>
            </div>

            <section id="contact">
                <div class="contact-section">
                    <h3 class="contact-header">"Correspondence"</h3>
                    <p class="contact-text">
                        "For inquiries, commissions, or simply to exchange pleasantries, do not hesitate to reach out through the following channels:"
                    </p>
                    <div class="contact-info">
                        <p>
                            "Electronic Mail: "
                            <a href="mailto:afkara@gmail.com">"afkara@gmail.com"</a>
                        </p>
                        <p>
                            "GitHub: " <a href="https://github.com/afkaralp" target="_blank">
                                "github.com/afkaralp"
                            </a>
                        </p>
                    </div>
                </div>
            </section>

            <Footer />
        </div>
    }
}

#[component]
fn ProjectCard(project: Project) -> impl IntoView {
    let owner = project.owner;
    let repo = project.repo;

    let stars = LocalResource::new(move || async move { fetch_github_stars(owner, repo).await });

    view! {
        <div class="project-card">
            <h4 class="project-title">
                <a href=project.url() target="_blank">
                    {project.name}
                </a>
            </h4>
            <p class="project-meta">
                <span class="project-language">{project.language}</span>
                <Suspense fallback=|| {
                    view! { <span class="project-stars">" · ★ ..."</span> }
                }>
                    {move || {
                        stars
                            .get()
                            .map(|result| {
                                match result {
                                    Some(count) => {
                                        view! {
                                            <span class="project-stars">" · ★ " {count}</span>
                                        }
                                            .into_any()
                                    }
                                    None => view! { <span class="project-stars"></span> }.into_any(),
                                }
                            })
                    }}
                </Suspense>
            </p>
            <p class="project-description">{project.description}</p>
        </div>
    }
}

#[component]
fn BlogPage() -> impl IntoView {
    let posts = BLOG_POSTS.clone();

    view! {
        <Title text="The Archives | The Afkara Gazette" />
        <Meta
            name="description"
            content="A collection of writings and musings on programming, data science, and technology."
        />
        <Meta property="og:title" content="The Archives | The Afkara Gazette" />
        <Meta
            property="og:description"
            content="A collection of writings and musings on programming, data science, and technology."
        />
        <Meta name="twitter:title" content="The Archives | The Afkara Gazette" />
        <Meta
            name="twitter:description"
            content="A collection of writings and musings on programming, data science, and technology."
        />

        <div class="page">
            <Header />

            <h2 class="headline">"The Archives"</h2>
            <p class="subheadline">"A Collection of Writings & Musings"</p>

            <div class="divider">
                <span class="divider-ornament">"§"</span>
            </div>

            <section class="blog-list">
                {posts
                    .iter()
                    .map(|post| {
                        let slug = post.slug.clone();
                        let slug2 = post.slug.clone();
                        view! {
                            <article class="blog-preview">
                                <h3 class="blog-preview-title">
                                    <a href=format!("/blog/{}", slug)>{post.title.clone()}</a>
                                </h3>
                                <p class="blog-preview-date">{post.date.clone()}</p>
                                <p class="blog-preview-excerpt">{get_excerpt(&post.content)}</p>
                                <a href=format!("/blog/{}", slug2) class="read-more">
                                    "Read More →"
                                </a>
                            </article>
                        }
                    })
                    .collect::<Vec<_>>()}
                {if posts.is_empty() {
                    Some(
                        view! {
                            <p class="article" style="text-align: center;">
                                "No dispatches have yet been published. Check back soon!"
                            </p>
                        },
                    )
                } else {
                    None
                }}
            </section>

            <Footer />
        </div>
    }
}

#[component]
fn BlogPostPage() -> impl IntoView {
    let params = leptos_router::hooks::use_params_map();

    let post = move || {
        let slug = params.read().get("slug");
        slug.and_then(|s| BLOG_POSTS.iter().find(|p| p.slug == s).cloned())
    };

    view! {
        <div class="page">
            <Header />

            {move || match post() {
                Some(p) => {
                    let title = format!("{} | The Afkara Gazette", p.title);
                    let og_title = p.title.clone();
                    let description = p.description.clone();
                    let og_description = p.description.clone();
                    let twitter_title = p.title.clone();
                    let twitter_description = p.description.clone();
                    let article_date = p.date.clone();

                    view! {
                        <Title text=title />
                        <Meta name="description" content=description />
                        <Meta property="og:type" content="article" />
                        <Meta property="og:title" content=og_title />
                        <Meta property="og:description" content=og_description />
                        <Meta property="article:published_time" content=article_date />
                        <Meta name="twitter:title" content=twitter_title />
                        <Meta name="twitter:description" content=twitter_description />

                        <article class="blog-post">
                            <h2 class="headline">{p.title.clone()}</h2>
                            <p class="subheadline">{p.date.clone()}</p>

                            <div class="divider">
                                <span class="divider-ornament">"§"</span>
                            </div>

                            <div class="blog-content" inner_html=render_markdown(&p.content) />
                        </article>

                        <div class="divider">
                            <span class="divider-ornament">"❧"</span>
                        </div>

                        <p style="text-align: center;">
                            <a href="/blog" class="back-link">
                                "← Return to Archives"
                            </a>
                        </p>
                    }
                        .into_any()
                }
                None => {
                    view! {
                        <Title text="Article Not Found | The Afkara Gazette" />

                        <h2 class="headline">"Article Not Found"</h2>
                        <p class="article" style="text-align: center;">
                            "The requested dispatch could not be located in our archives."
                        </p>
                        <p style="text-align: center;">
                            <a href="/blog" class="back-link">
                                "← Return to Archives"
                            </a>
                        </p>
                    }
                        .into_any()
                }
            }}

            <Footer />
        </div>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <Title text="Page Not Found | The Afkara Gazette" />

        <div class="page">
            <Header />

            <h2 class="headline">"Page Not Found"</h2>
            <p class="subheadline">"Error 404"</p>

            <div class="divider">
                <span class="divider-ornament">"§"</span>
            </div>

            <p class="article" style="text-align: center;">
                "Alas! The page you seek has been lost to the annals of time, or perhaps never existed at all."
            </p>

            <p style="text-align: center;">
                <a href="/" class="back-link">
                    "← Return to the Front Page"
                </a>
            </p>

            <Footer />
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
