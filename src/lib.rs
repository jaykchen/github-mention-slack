use airtable_flows::create_record;
use chrono::DateTime;
use dotenv::dotenv;
use github_flows::{
    listen_to_event,
    octocrab::models::events::payload::{
        IssueCommentEventAction, IssuesEventAction, PullRequestEventAction,
    },
    EventPayload,
    GithubLogin::Provided,
};
use slack_flows::send_message_to_channel;
use std::env;

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn run() -> anyhow::Result<()> {
    dotenv().ok();

    // flows function watches your [github_login] for mentions in Issues, PR, and comments
    let github_login = env::var("github_login").unwrap_or("alabulei1".to_string());
    let github_owner = env::var("github_owner").unwrap_or("alabulei1".to_string());
    let github_repo = env::var("github_repo").unwrap_or("a-test".to_string());

    listen_to_event(
        &Provided(github_login.clone()),
        &github_owner,
        &github_repo,
        vec![
            "issues",
            "pull_request",
            "issue_comment",
            "pull_request_review",
            "pull_request_review_comment",
        ],
        |payload| handler(&github_login, payload),
    )
    .await;

    Ok(())
}

async fn handler(login: &str, payload: EventPayload) {
    let slack_workspace = env::var("slack_workspace").unwrap_or("secondstate".to_string());
    let slack_channel = env::var("slack_channel").unwrap_or("github-status".to_string());

    let airtable_token_name = env::var("airtable_token_name").unwrap_or("github".to_string());
    let airtable_base_id = env::var("airtable_base_id").unwrap_or("appNEswczILgUsxML".to_string());
    let airtable_table_name = env::var("airtable_table_name").unwrap_or("fork".to_string());

    let at_string = format!("@{}", login);
    let mut is_mentioned = false;
    let mut is_valid_event = true;
    let mut name = "".to_string();
    let mut time = DateTime::default();
    let mut title = "".to_string();
    let mut html_url = "".to_string();

    match payload {
        EventPayload::IssuesEvent(e) => {
            let issue = e.issue;
            let action = e.action;
            is_mentioned = issue.body.unwrap_or("".to_string()).contains(&at_string);

            is_valid_event = action != IssuesEventAction::Closed;

            if is_mentioned && is_valid_event {
                name = issue.user.login;
                title = issue.title;
                html_url = issue.html_url.to_string();
                time = issue.created_at;
            }
        }

        EventPayload::IssueCommentEvent(e) => {
            let comment = e.comment;
            let action = e.action;
            is_mentioned = comment.body.unwrap_or("".to_string()).contains(&at_string);
            is_valid_event = action != IssueCommentEventAction::Deleted;
            if is_mentioned && is_valid_event {
                name = comment.user.login;
                title = e.issue.title;
                html_url = comment.html_url.to_string();
                time = comment.created_at;
            }
        }

        EventPayload::PullRequestEvent(e) => {
            let pull_request = e.pull_request;
            let action = e.action;
            is_mentioned = pull_request
                .body
                .unwrap_or("".to_string())
                .contains(&at_string);
            is_valid_event = action != PullRequestEventAction::Closed;
            if is_mentioned && is_valid_event {
                name = pull_request.user.unwrap().login;
                title = pull_request.title.unwrap();
                html_url = pull_request
                    .html_url
                    .expect("html_url not found")
                    .to_string();
                time = pull_request.created_at.unwrap();
            }
        }

        EventPayload::PullRequestReviewEvent(e) => {
            let review = e.review;
            is_mentioned = review.body.unwrap_or("".to_string()).contains(&at_string);

            if is_mentioned {
                name = review.user.unwrap().login;
                title = e.pull_request.title.unwrap();
                html_url = review.html_url.to_string();
                time = review.submitted_at.unwrap();
            }
        }

        EventPayload::PullRequestReviewCommentEvent(e) => {
            let comment = e.comment;
            is_mentioned = comment.body.unwrap_or("".to_string()).contains(&at_string);

            if is_mentioned {
                name = comment.user.login;
                title = e.pull_request.title.unwrap();
                html_url = comment.html_url.to_string();
                time = comment.created_at;
            }
        }

        _ => (),
    }

    if is_valid_event && is_mentioned {
        let text = format!("{name} mentioned you in {title}\n{html_url}");
        send_message_to_channel(&slack_workspace, &slack_channel, text);

        let data = serde_json::json!({
        "Name": name,
        "Repo": html_url,
        "Created": time,
        });
        create_record(
            &airtable_token_name,
            &airtable_base_id,
            &airtable_table_name,
            data,
        )
    }
}
