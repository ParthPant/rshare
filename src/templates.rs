use rust_embed::RustEmbed;
use tera::{Context, Tera};

#[derive(RustEmbed)]
#[folder = "templates/"]
struct Assets;

pub struct Templates {
    tera: Tera,
}

impl Templates {
    pub fn new() -> Self {
        let mut tera = Tera::default();

        for file in Assets::iter() {
            let name = file.as_ref();
            log::debug!("{}", name);

            let content = Assets::get(name).unwrap();
            let content = std::str::from_utf8(content.data.as_ref()).unwrap();
            log::debug!("{}", content);

            tera.add_raw_template(name, content)
                .expect("Failed to add template");
        }

        Templates { tera }
    }

    pub fn render(&self, name: &str, ctx: &Context) -> Result<String, tera::Error> {
        self.tera.render(name, &ctx)
    }
}
