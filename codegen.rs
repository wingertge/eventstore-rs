// extern crate protoc_rust;
// extern crate tonic_build;

// use protoc_rust::Customize;

// fn main() {
//     protoc_rust::run(protoc_rust::Args {
//         out_dir: "src/internal",
//         input: &["protos/messages.proto"],
//         includes: &["protos"],
//         customize: Customize {
//             carllerche_bytes_for_bytes: Some(true),
//             carllerche_bytes_for_string: Some(true),
//             ..Default::default()
//         },
//     }).expect("protoc");
// }
//
fn main() {
    // let files = [
    //     "protos/es6/persistent.proto",
    //     "protos/es6/projections.proto",
    //     "protos/es6/streams.proto",
    //     "protos/es6/users.proto",
    // ];

    // tonic_build::configure()
    //     .build_server(false)
    //     .out_dir("src/es6/grpc")
    //     .compile(&files, &["protos/es6"]).unwrap();
    tonic_build::compile_protos("protos/es6/persistent.proto").unwrap();
    tonic_build::compile_protos("protos/es6/projections.proto").unwrap();
    tonic_build::compile_protos("protos/es6/streams.proto").unwrap();
    tonic_build::compile_protos("protos/es6/users.proto").unwrap();
}
