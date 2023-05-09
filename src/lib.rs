use dotenv::dotenv;
use github_flows::{get_octo, listen_to_event, EventPayload, GithubLogin::Provided};
use openai_flows::chat::{ChatModel, ChatOptions};
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
        vec!["issues"],
        |payload| handler(&github_login, payload),
    )
    .await;

    Ok(())
}

async fn handler(login: &str, payload: EventPayload) {
    let openai_key_name = env::var("openai_key_name").unwrap_or("secondstate".to_string());
    let slack_workspace = env::var("slack_workspace").unwrap_or("secondstate".to_string());
    let slack_channel = env::var("slack_channel").unwrap_or("github-status".to_string());

    match payload {
        EventPayload::UnknownEvent(e) => {
            let action = e.action;
            if action == "notify" {
                let octocrab = get_octo(&Provided(login.to_string()));
                let activity = octocrab.activity();
                match activity.notifications().list().send().await {
                    Ok(notes) => {
                        for note in notes {
                            if note.unread && note.reason == "mention" {
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
        _ => (),
    }
}
