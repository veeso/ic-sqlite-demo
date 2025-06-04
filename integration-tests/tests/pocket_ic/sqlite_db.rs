use candid::Encode;
use did::User;
use integration_tests::TestEnv as _;
use integration_tests::actor::admin;

#[pocket_test::test]
async fn test_should_insert_and_query(env: PocketIcTestEnv) {
    let canister = env.sqlite_db();
    let new_user = User {
        id: 1,
        name: "Alice".to_string(),
        age: 30,
    };

    // create
    let res = env
        .update::<()>(canister, admin(), "create", Encode!(&()).unwrap())
        .await;
    assert!(res.is_ok(), "Failed to create user: {:?}", res);

    // create user
    let res = env
        .update::<()>(
            canister,
            admin(),
            "insert_user",
            Encode!(&new_user.clone()).unwrap(),
        )
        .await;
    assert!(res.is_ok(), "Failed to insert user: {:?}", res);

    let res = env
        .query::<User>(canister, admin(), "get_user", Encode!(&0u64).unwrap())
        .await
        .expect("Failed to get user");

    assert_eq!(res, new_user, "Failed to get user: {:?}", res);
}
