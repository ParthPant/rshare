use tera::{Tera, Context};

pub struct Templates {
    tera : Tera
}

impl Templates {
    pub fn new(path: &str) -> Self {
        let tera = match Tera::new(path) {
            Ok(t) => t,
            Err(e) => {
                log::error!("Tera Parsing Error: {}", e);
                panic!();
            }
        };

        Templates { tera }
    }

    pub fn render(&self, name: &str, ctx: &Context) -> Result<String, tera::Error> {
        self.tera.render(name, &ctx)
    }
}
