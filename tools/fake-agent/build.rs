fn main() {
    prost_build::compile_protos(&["../../proto/sentinel.proto"], &["../../proto"]).unwrap();
}
