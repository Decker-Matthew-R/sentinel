fn main() {
    slint_build::compile("ui/main.slint").unwrap();
    prost_build::compile_protos(&["../proto/sentinel.proto"], &["../proto"]).unwrap();
}
