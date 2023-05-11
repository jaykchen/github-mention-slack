use dotenv::dotenv;
use github_flows::{listen_to_event, EventPayload, GithubLogin::Provided};
use slack_flows::send_message_to_channel;
use std::env;

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn run() -> anyhow::Result<()> {
    dotenv().ok();
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
        |payload| handler(&github_owner, payload),
    )
    .await;

    Ok(())
}

async fn handler(owner: &str, payload: EventPayload) {
    let slack_workspace = env::var("slack_workspace").unwrap_or("secondstate".to_string());
    let slack_channel = env::var("slack_channel").unwrap_or("github-status".to_string());

    let at_string = format!("@{}", owner);
    let mut is_mentioned = false;
    let mut title = "".to_string();
    let mut html_url = "".to_string();

    match payload {
        EventPayload::IssuesEvent(e) => {
            let issue = e.issue;
            is_mentioned = issue.body.unwrap_or("".to_string()).contains(&at_string);

            if is_mentioned {
                title = issue.title;
                html_url = issue.html_url.to_string();
            }
        }

        EventPayload::IssueCommentEvent(e) => {
            let comment = e.comment;
            is_mentioned = comment.body.unwrap_or("".to_string()).contains(&at_string);

            if is_mentioned {
                title = e.issue.title;
                html_url = comment.html_url.to_string();
            }
        }

        EventPayload::PullRequestEvent(e) => {
            let pull_request = e.pull_request;
            is_mentioned = pull_request
                .body
                .unwrap_or("".to_string())
                .contains(&at_string);
            if is_mentioned {
                title = pull_request.title.unwrap();
                html_url = pull_request
                    .html_url
                    .expect("html_url not found")
                    .to_string();
            }
        }

        EventPayload::PullRequestReviewEvent(e) => {
            let review = e.review;
            is_mentioned = review.body.unwrap_or("".to_string()).contains(&at_string);
            if is_mentioned {
                title = e.pull_request.title.unwrap();
                html_url = review.html_url.to_string();
            }
        }

        EventPayload::PullRequestReviewCommentEvent(e) => {
            let comment = e.comment;
            is_mentioned = comment.body.unwrap_or("".to_string()).contains(&at_string);

            if is_mentioned {
                title = e.pull_request.title.unwrap();
                html_url = comment.html_url.to_string();
            }
        }

        _ => (),
    }

    if is_mentioned {
        let text = format!("You are mentioned in:\n{title}\n{html_url}");
        send_message_to_channel(&slack_workspace, &slack_channel, text);
    }
}
