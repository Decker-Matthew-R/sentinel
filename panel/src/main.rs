mod framing;
mod proto;

slint::include_modules!();

fn main() {
    AppWindow::new().unwrap().run().unwrap();
}

#[cfg(test)]
mod tests {
    #[test]
    fn panel_tests_run() {}
}
