// use crate::api::server::server::QueryResponse;
// use wiremock_grpc::generate;
// use wiremock_grpc::*;
//
// generate!("EcdarBackend", MyMockServer);

#[ignore]
#[tokio::test]
async fn send_query_test_correct_query_returns_ok() {
    //todo!("Somehow QueryResponse does not implement prost::message::Message even though it does.
    // supposedly a versioning error between wiremock_grpc, tonic, and prost")

    // let mut server = MyMockServer::start_default().await;
    //
    // let request1 = server.setup(
    //     MockBuilder::when()
    //         .path("EcdarBackend/SendQuery")
    //         .then()
    //         .return_status(Code::Ok)
    //         .return_body(|| QueryResponse {
    //             query_id: 0,
    //             info: vec![],
    //             result: None,
    //         }),
    // );

    //...
    //https://crates.io/crates/wiremock-grpc
}
