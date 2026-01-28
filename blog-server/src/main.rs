mod blog_grpc {
    tonic::include_proto!("blog");
}

fn main() {
    blog_grpc::Post{};
    println!("Hello, world!");
}
