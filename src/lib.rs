use dotenv::dotenv;
use github_flows::{get_octo, listen_to_event, EventPayload, GithubLogin::Provided};
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
        |payload| handler(&github_login, payload),
    )
    .await;

    Ok(())
}

async fn handler(login: &str, payload: EventPayload) {
    let slack_workspace = env::var("slack_workspace").unwrap_or("secondstate".to_string());
    let slack_channel = env::var("slack_channel").unwrap_or("github-status".to_string());

    let mut issue = None;
    let mut pull_request = None;

    match payload {
        EventPayload::IssuesEvent(e) => {
            issue = Some(e.issue.clone());
        }

        EventPayload::IssueCommentEvent(e) => {
            issue = Some(e.issue.clone());
            send_message_to_channel(&slack_workspace, &slack_channel, e.issue.title.clone());
        }

        EventPayload::PullRequestEvent(e) => {
            pull_request = Some(e.pull_request.clone());
        }

        EventPayload::PullRequestReviewEvent(e) => {
            pull_request = Some(e.pull_request.clone());
            send_message_to_channel(
                &slack_workspace,
                &slack_channel,
                e.pull_request.title.unwrap(),
            );
        }

        EventPayload::PullRequestReviewCommentEvent(e) => {
            pull_request = Some(e.pull_request.clone());
            send_message_to_channel(
                &slack_workspace,
                &slack_channel,
                e.pull_request.title.unwrap(),
            );
        }

        _ => (),
    }

    if issue.is_some() || pull_request.is_some() {
        let octocrab = get_octo(&Provided(login.to_string()));
        let activity = octocrab.activity();

        match activity.notifications().list().send().await {
            Ok(notes) => {
                send_message_to_channel(
                    &slack_workspace,
                    &slack_channel,
                    notes.clone().total_count.unwrap().to_string(),
                );

                for note in notes.items {
                    if note.reason == "mention" || note.reason == "subscribed" {
                        let title = note.subject.title;
                        let html_url = &note.subject.url.unwrap();
                        let text = format!("{title}\n{html_url}");
                        send_message_to_channel(&slack_workspace, &slack_channel, text);
                    }
                }
            }
            Err(_e) => {}
        };
    }
}
