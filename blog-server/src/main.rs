mod domain;

mod blog_grpc {
    tonic::include_proto!("blog");
}

fn main() {
    println!("Hello, world!");
}
