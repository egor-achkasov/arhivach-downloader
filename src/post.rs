use crate::file::File;

/// Represents a single post in a thread
#[derive(Debug, Clone)]
pub struct Post {
    /// Empty if None
    pub subject: Option<String>,
    /// "Аноним" if none
    pub name: Option<String>,
    /// "mailto:sage"
    pub mailto: Option<String>,
    /// "01/02/26 Вск 03:13:12"
    pub time: String,
    /// "#5"
    pub num: String,
    /// "329281515"
    pub id: u32,
    pub files: Vec<File>,
    /// Post text
    pub text: String,
}

impl Post {
    pub fn parse_posts(
        html: &str,
    ) -> Result<Vec<Post>, Box<dyn std::error::Error>> {
        let mut posts = Vec::new();
        
        let document = scraper::Html::parse_document(html);
        let selector = scraper::Selector::parse(r#"div.post"#).unwrap();
        for node in document.select(&selector) {
            let post = Post::parse_post(node)?;
            posts.push(post);
        }

        Ok(posts)
    }

    /// Parse div class="post"
    /// 
    /// Example element:
    /// ```html
    /// <div class="post" id="post329274763" postid="329274763">
    ///     <div class="post_head">...</div> (see parse_post_head function)
    ///     <span class="post_comment">...</span> (see parse_post_comment function)
    /// </div>
    /// ```
    fn parse_post(node: scraper::ElementRef) -> Result<Post, Box<dyn std::error::Error>> {
        static SEL_POST_HEAD: std::sync::LazyLock<scraper::Selector> = std::sync::LazyLock::new(
            || scraper::Selector::parse("div.post_head").unwrap()
        );
        static SEL_POST_IMAGE_BLOCK: std::sync::LazyLock<scraper::Selector> = std::sync::LazyLock::new(
            || scraper::Selector::parse("span.post_comment").unwrap()
        );

        let post_head = node
            .select(&SEL_POST_HEAD)
            .next()
            .ok_or("missing post_head")?;
        let (subject, name, mailto, time, num, id) = Post::parse_post_head(post_head)?;

        let post_comment = node
            .select(&SEL_POST_IMAGE_BLOCK)
            .next()
            .ok_or("missing post_comment")?;
        let (files, text) = Post::parse_post_comment(post_comment)?;

        Ok(Post {
            subject,
            name,
            mailto,
            time,
            num,
            id,
            files,
            text,
        })
    }

    /// Parses the post_head element
    ///
    /// Returns (subject, name, mailto, time, num, id)
    /// Returns error if no time, num or id is found or if id is not a number
    /// 
    /// Example element:
    /// ```html
    /// <div class="post_head">
    ///     <span class="poster_name" title="">Аноним</span>&nbsp;
    ///     <span class="post_time">01/02/26 Вск 04:27:32</span>&nbsp;
    ///     <span class="post_num">#77</span>&nbsp;
    ///     <span class="post_id">
    ///         <a style="position:absolute;margin-top:-50px;" id="329274763"></a>
    ///         <a href="#329274763">№329274763</a>
    ///     </span> &nbsp;
    /// </div>
    /// ```
    fn parse_post_head(
        post_head: scraper::ElementRef
    ) -> Result<
        (
            Option<String>, // subject
            Option<String>, // name
            Option<String>, // mailto
            String,         // time
            String,         // num
            u32             // id
        ),
        Box<dyn std::error::Error>
    > {
        let sel = |s| scraper::Selector::parse(s).unwrap();

        let id: u32 = post_head
            .select(&sel("span.post_id a[href]"))
            .next()
            .and_then(|el| el.value().attr("href"))
            .and_then(|href| href.strip_prefix('#'))
            .ok_or("missing post id")?
            .parse()?;

        let subject = post_head
            .select(&sel("h1.post_subject"))
            .next()
            .map(|el| el.text().collect::<String>());

        let name = post_head
            .select(&sel("span.poster_name"))
            .next()
            .map(|el| el.text().collect::<String>())
            .and_then(|n| if n == "Аноним" { None } else { Some(n) });

        let mailto = post_head
            .select(&sel("a.post_mail"))
            .next()
            .and_then(|el| el.value().attr("title"))
            .map(|s| s.to_string());

        let time = post_head
            .select(&sel("span.post_time"))
            .next()
            .ok_or("missing post_time")?
            .text()
            .collect::<String>();

        let num = post_head
            .select(&sel("span.post_num"))
            .next()
            .ok_or("missing post_num")?
            .text()
            .collect::<String>();

        Ok((subject, name, mailto, time, num, id))
    }

    /// Parses the sapn post_comment element from a post element
    /// 
    /// Returns (files, text)
    /// 
    /// Example element:
    /// <span class="post_comment">
    ///     <div class="post_image_block" ...>...</div> (see parse_post_image_block function) (can appear 0 to multiple times)
    ///     <div class="post_comment_body">...</div> (see parse_post_comment_body function)
    /// </span>
    fn parse_post_comment(
        node: scraper::ElementRef,
    ) -> Result<(Vec<File>, String), Box<dyn std::error::Error>> {
        static SEL_POST_IMAGE_BLOCK: std::sync::LazyLock<scraper::Selector> = std::sync::LazyLock::new(
            || scraper::Selector::parse("div.post_image_block").unwrap()
        );
        static SEL_POST_COMMENT_BODY: std::sync::LazyLock<scraper::Selector> = std::sync::LazyLock::new(
            || scraper::Selector::parse("div.post_comment_body").unwrap()
        );

        // TODO handle the errors instead of propagating them upper. Change the return type to non-Result
        let files: Vec<File> = node
            .select(&SEL_POST_IMAGE_BLOCK)
            .map(Post::parse_post_image_block)
            .collect();
        let text = Post::parse_post_comment_body(node
            .select(&SEL_POST_COMMENT_BODY)
            .next()
            .ok_or("missing post_comment_body")?);
        Ok((files, text))
    }

    /// Parses "post_image_block" element
    /// Returns File
    /// 
    /// Example element:
    /// ```html
    /// <div class="post_image_block" id="pib_77_2" pib="77_2" title="537.4 Кб, 946 x 946
    /// image.png
    /// 17699092523481.png">
    ///     <a class="expand_image" onclick="expand_local('77_2','/storage/a/cc/acc7f5856bc60ad3bdbd4dc7027e33f9.png','946','946',event); return false;" href="#">
    ///         <div class="post_image" id="thumb_77_2">
    ///             <img src="/storage/t/acc7f5856bc60ad3bdbd4dc7027e33f9.png" alt="" loading="lazy"> // thumbnail path
    ///         </div>
    ///     </a>
    ///     <a href="/storage/a/cc/acc7f5856bc60ad3bdbd4dc7027e33f9.png" target="_blank" class="img_filename">image.png</a> // can also be https://i.arhivach.vc/... if it's a video
    /// </div>
    /// ```
    fn parse_post_image_block(pib: scraper::ElementRef) -> File {
        static SEL_POST_IMAGE_IMG: std::sync::LazyLock<scraper::Selector> = std::sync::LazyLock::new(
            || scraper::Selector::parse(".post_image img").unwrap()
        );
        static SEL_A_IMG_FILENAME: std::sync::LazyLock<scraper::Selector> = std::sync::LazyLock::new(
            || scraper::Selector::parse("a.img_filename").unwrap()
        );

        // Title example:
        // 402.2 Кб, 800 x 532
        // image.png <- name_orig
        // 17699142349880.png <- name_timestamp
        let title = pib.value().attr("title").unwrap_or("");
        let title_lines: Vec<&str> = title.lines().collect();
        let name_orig      = title_lines
            .get(1)
            .map(|s| s.to_string())
            .unwrap_or("unnamed".to_string());
        let name_timestamp = title_lines
            .get(2)
            .map(|s| s.to_string())
            .unwrap_or("unnamed".to_string());

        // url_thumb
        let url_thumb = pib
            .select(&SEL_POST_IMAGE_IMG)
            .next()
            .and_then(|el| el.value().attr("src"))
            .unwrap_or(""); // /storage/t/83c2fe5ba9a8469d9eeef4af124e3b52.thumb
        let url_thumb = if url_thumb.is_empty() {
            String::new()
        } else {
            format!("https://arhivach.vc{}", url_thumb)
        };

        // url
        let url = pib
            .select(&SEL_A_IMG_FILENAME)
            .next()
            .and_then(|el| el.value().attr("href"))
            .unwrap_or("");
        let url = if url.starts_with("http") { // is `https://i.arhivach.vc/...`?
            url.to_string()
        } else if url.is_empty() {
            String::new()
        } else {
            format!("https://arhivach.vc{}", url)
        };

        File {
            name_orig,
            name_timestamp,
            url_thumb,
            url,
        }
    }

    /// Parses the post text from `div.post_comment_body`
    ///
    /// Returns post text:
    /// - References are plaintext (e.g. >>329274789)
    /// - `<br>` is replaced with \n
    /// - `<span class="unkfunc">` (greentext) is replaced with >text
    /// 
    /// If the text contains a reference (e.g. >>329274789) it looks like this in the element:
    /// ```html
    /// <div class="post_comment_body">
    ///     <a href="#329274893" class="post-reply-link" data-thread="329273515" data-num="329274893">&gt;&gt;329274893</a> // This will be replaced with >>329274893
    ///     <br>
    ///     <span class="unkfunc">&gt;greentext1</span>
    ///     <br>
    ///     text1
    /// </div>
    /// ```
    /// 
    /// This example returns:
    /// ```text
    /// >>329274893
    /// >greentext1
    /// text1
    /// ```
    fn parse_post_comment_body(node: scraper::ElementRef) -> String {
        use scraper::node::Node;

        let mut result = String::new();
        for child in node.children() {
            match child.value() {
                Node::Text(text) => result.push_str(&text.text),
                Node::Element(el) if el.name() == "br" => result.push('\n'),
                Node::Element(_) => {
                    if let Some(el_ref) = scraper::ElementRef::wrap(child) {
                        result.push_str(&el_ref.text().collect::<String>());
                    }
                }
                _ => {}
            }
        }
        result.trim().to_string()
    }
}

impl std::fmt::Display for Post {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Header line
        let name = self.name.as_deref().unwrap_or("Аноним");
        let mailto = self.mailto.as_deref().unwrap_or("");
        
        if !mailto.is_empty() {
            write!(f, "{} ({})", name, mailto)?;
        } else {
            write!(f, "{}", name)?;
        }
        
        write!(f, " {} {} ID:{}", self.time, self.num, self.id)?;
        
        // Subject
        if let Some(ref subject) = self.subject {
            write!(f, "\n{}", subject)?;
        }
        
        // Files
        if !self.files.is_empty() {
            write!(f, "\n[Files: {}]", self.files.len())?;
            for file in &self.files {
                write!(f, "\n  - {}", file)?;
            }
        }
        
        // Post text
        if !self.text.is_empty() {
            write!(f, "\n{}", self.text)?;
        }
        
        Ok(())
    }
}
