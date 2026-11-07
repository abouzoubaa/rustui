//! Validation examples using the public RustUI field API.
use rustui::prelude::*;

pub fn demo() -> impl View {
    Component("validation-gallery", ValidationGallery::default)
}

#[component]
struct ValidationGallery {
    #[state(String::new())]
    email: String,
    #[state(String::new())]
    password: String,
    #[state(String::new())]
    query: String,
    #[state(false)]
    email_valid: bool,
    #[state(false)]
    password_valid: bool,
    #[state(false)]
    query_valid: bool,
}

impl View for ValidationGallery {
    fn body(&self) -> impl View {
        VStack::new()
            .spacing(10.0)
            .child(Text("Validation"))
            .child(
                TextField("Email", self.email_binding())
                    .id("validation-email")
                    .validate(|value| {
                        if value.contains('@') {
                            Ok(())
                        } else {
                            Err("Enter an email containing @".into())
                        }
                    })
                    .is_valid(self.email_valid_binding()),
            )
            .child(Text(format!("Email valid: {}", self.email_valid())))
            .child(
                SecureField("Password", self.password_binding())
                    .id("validation-password")
                    .validate(|value| {
                        if value.chars().count() >= 8 {
                            Ok(())
                        } else {
                            Err("Use at least 8 characters".into())
                        }
                    })
                    .is_valid(self.password_valid_binding()),
            )
            .child(Text(format!("Password valid: {}", self.password_valid())))
            .child(
                SearchField("Search", self.query_binding())
                    .id("validation-search")
                    .validate(|value| {
                        if value.trim().is_empty() {
                            Err("Enter a search term".into())
                        } else {
                            Ok(())
                        }
                    })
                    .is_valid(self.query_valid_binding()),
            )
            .child(Text(format!("Search valid: {}", self.query_valid())))
    }
}
