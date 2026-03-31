//! 博文管理接口相关的集成测试。
//!
//! 这里重点验证两类行为：
//! - 博文接口是否按 scope 控制读写能力
//! - Markdown 内容是否可以完整创建、读取、更新与删除

use app_testkit::{request_empty_json_with_auth, request_json_with_auth};
use http::{Method, StatusCode};
use serde_json::json;
use uuid::Uuid;

mod support;

#[tokio::test]
/// 验证普通用户可读博文但不能创建博文。
async fn posts_routes_should_enforce_scopes() {
    let Some((app, _config, db)) = support::setup_app_with_db().await else {
        return;
    };

    let session = support::register_and_login(
        app.clone(),
        "post-reader",
        &format!("post-reader-{}@example.com", Uuid::new_v4()),
        "secret123",
    )
    .await;

    let (list_status, list_body) = request_empty_json_with_auth(
        app.clone(),
        Method::GET,
        "/posts",
        Some(&session.access_token),
    )
    .await;
    assert_eq!(list_status, StatusCode::OK, "list body: {list_body}");

    let (create_status, create_body) = request_json_with_auth(
        app.clone(),
        Method::POST,
        "/posts",
        json!({
            "title": "first post",
            "slug": format!("first-post-{}", Uuid::new_v4()),
            "content_md": "# hello\n\ncontent",
            "published": true
        }),
        Some(&session.access_token),
    )
    .await;
    assert_eq!(create_status, StatusCode::FORBIDDEN, "body: {create_body}");
    assert_eq!(
        create_body["error_code"].as_str(),
        Some("AUTH_FORBIDDEN_SCOPE")
    );

    support::delete_users(&db, &[session.user_id]).await;
}

#[tokio::test]
/// 验证管理员可以完整管理 Markdown 博文。
async fn admin_should_manage_markdown_posts() {
    let Some((app, _config, db)) = support::setup_app_with_db().await else {
        return;
    };

    let admin_session = support::register_and_login(
        app.clone(),
        "post-admin",
        &format!("post-admin-{}@example.com", Uuid::new_v4()),
        "secret123",
    )
    .await;
    let admin_session = support::promote_to_admin(app.clone(), &db, admin_session).await;

    let slug = format!("markdown-post-{}", Uuid::new_v4());
    let markdown = "# Heading\n\n- item 1\n- item 2\n\n```md\ncode\n```";
    let (create_status, create_body) = request_json_with_auth(
        app.clone(),
        Method::POST,
        "/posts",
        json!({
            "title": "Markdown Post",
            "slug": slug,
            "content_md": markdown,
            "published": true
        }),
        Some(&admin_session.access_token),
    )
    .await;
    assert_eq!(create_status, StatusCode::OK, "body: {create_body}");
    assert_eq!(
        create_body["data"]["content_md"].as_str(),
        Some(markdown),
        "body: {create_body}"
    );
    let post_id = create_body["data"]["id"]
        .as_str()
        .expect("missing post id")
        .parse::<Uuid>()
        .expect("invalid post id");

    let (get_status, get_body) = request_empty_json_with_auth(
        app.clone(),
        Method::GET,
        &format!("/posts/{post_id}"),
        Some(&admin_session.access_token),
    )
    .await;
    assert_eq!(get_status, StatusCode::OK, "body: {get_body}");
    assert_eq!(
        get_body["data"]["slug"].as_str(),
        create_body["data"]["slug"].as_str()
    );

    let (get_by_slug_status, get_by_slug_body) = request_empty_json_with_auth(
        app.clone(),
        Method::GET,
        &format!("/posts/slug/{slug}"),
        Some(&admin_session.access_token),
    )
    .await;
    assert_eq!(
        get_by_slug_status,
        StatusCode::OK,
        "body: {get_by_slug_body}"
    );
    assert_eq!(
        get_by_slug_body["data"]["id"].as_str(),
        Some(post_id.to_string().as_str())
    );

    let updated_markdown = "## Updated\n\nThis is still **markdown**.";
    let updated_slug = format!("updated-markdown-post-{}", Uuid::new_v4());
    let (update_status, update_body) = request_json_with_auth(
        app.clone(),
        Method::PUT,
        &format!("/posts/{post_id}"),
        json!({
            "title": "Updated Markdown Post",
            "slug": updated_slug,
            "content_md": updated_markdown,
            "published": false
        }),
        Some(&admin_session.access_token),
    )
    .await;
    assert_eq!(update_status, StatusCode::OK, "body: {update_body}");
    assert_eq!(
        update_body["data"]["content_md"].as_str(),
        Some(updated_markdown),
        "body: {update_body}"
    );
    assert_eq!(update_body["data"]["published"].as_bool(), Some(false));

    let (list_status, list_body) = request_empty_json_with_auth(
        app.clone(),
        Method::GET,
        "/posts?sort=title&order=asc&filter=updated-markdown-post&published=false",
        Some(&admin_session.access_token),
    )
    .await;
    assert_eq!(list_status, StatusCode::OK, "body: {list_body}");
    assert_eq!(list_body["data"]["total"].as_i64(), Some(1));
    assert_eq!(
        list_body["data"]["items"][0]["id"].as_str(),
        Some(post_id.to_string().as_str())
    );

    let (published_list_status, published_list_body) = request_empty_json_with_auth(
        app.clone(),
        Method::GET,
        "/posts?published=true",
        Some(&admin_session.access_token),
    )
    .await;
    assert_eq!(
        published_list_status,
        StatusCode::OK,
        "body: {published_list_body}"
    );
    assert_eq!(published_list_body["data"]["total"].as_i64(), Some(0));

    let (delete_status, delete_body) = request_empty_json_with_auth(
        app.clone(),
        Method::DELETE,
        &format!("/posts/{post_id}"),
        Some(&admin_session.access_token),
    )
    .await;
    assert_eq!(delete_status, StatusCode::OK, "body: {delete_body}");

    let (missing_status, missing_body) = request_empty_json_with_auth(
        app,
        Method::GET,
        &format!("/posts/{post_id}"),
        Some(&admin_session.access_token),
    )
    .await;
    assert_eq!(
        missing_status,
        StatusCode::NOT_FOUND,
        "body: {missing_body}"
    );

    support::delete_posts(&db, &[post_id]).await;
    support::delete_users(&db, &[admin_session.user_id]).await;
}
