use std::{cell::RefCell, error::Error, io::Write, sync::Arc, vec};

use chrono::{Month, NaiveDate, Utc};
use num_traits::FromPrimitive;

use utils::{prompt_activity_select, prompt_task_select, render_table};

use crate::moco::model::{ControlActivityTimer, CreateActivity, DeleteActivity, GetActivity};
use crate::utils::{prompt_activity_select_date, prompt_activity_select_today};
use crate::{
    moco::{client::MocoClient, model::EditActivity},
    utils::{ask_question, mandatory_validator},
};

mod cli;
mod config;
mod moco;

mod utils;

const FORMAT_DATE: &str = "%Y-%m-%d";
const FORMAT_DATE_DAY: &str = "%Y-%m-%d, %A";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::init();
    let mut log_builder = env_logger::builder();
    log_builder.parse_default_env();
    if args.debug {
        log_builder.filter_level(log::LevelFilter::Trace);
    }
    log_builder.init();
    let config = Arc::new(RefCell::new(config::init()?));
    let moco_client = MocoClient::new(&config);

    match args.command {
        cli::Commands::Login { system } => match system {
            cli::Login::Moco => {
                println!("Moco Login");

                let moco_company = ask_question("Enter Moco company name: ", &mandatory_validator)?;
                let api_key = ask_question("Enter your personal API key: ", &mandatory_validator)?;
                let bot_api_key =
                    ask_question("Enter the Moco Bot API key: ", &mandatory_validator)?;

                config.borrow_mut().moco_company = Some(moco_company);
                config.borrow_mut().moco_api_key = Some(api_key);
                config.borrow_mut().moco_bot_api_key = Some(bot_api_key);

                let firstname = ask_question("Enter firstname: ", &mandatory_validator)?;
                let lastname = ask_question("Enter lastname:  ", &mandatory_validator)?;

                let client_id = moco_client.get_user_id(firstname, lastname).await?;

                config.borrow_mut().moco_user_id = client_id;
                config.borrow_mut().write_config()?;
                println!("🤩 Logged in 🤩")
            }
        },
        cli::Commands::List {
            week,
            month,
            backward,
            date,
        } => {
            let activities = match date {
                Some(date) => {
                    println!("List activities for {}", date.format(FORMAT_DATE_DAY));
                    let date_string = date.format(FORMAT_DATE).to_string();
                    moco_client.get_activities(date_string.clone(), date_string, None, None)
                }
                None => {
                    let (from, to) = utils::select_from_to_date(week, month, backward);
                    println!(
                        "List activities from {} – {}",
                        from.format(FORMAT_DATE_DAY),
                        to.format(FORMAT_DATE_DAY)
                    );
                    moco_client.get_activities(
                        from.format(FORMAT_DATE).to_string(),
                        to.format(FORMAT_DATE).to_string(),
                        None,
                        None,
                    )
                }
            }
            .await?;

            let mut list: Vec<Vec<String>> = activities
                .iter()
                .map(|activity| {
                    vec![
                        activity.date.clone(),
                        NaiveDate::parse_from_str(&activity.date, FORMAT_DATE)
                            .unwrap()
                            .format("%A")
                            .to_string(),
                        activity.hours.to_string(),
                        activity.customer.name.clone(),
                        activity.task.name.clone(),
                        activity
                            .description
                            .as_ref()
                            .unwrap_or(&String::new())
                            .to_string(),
                    ]
                })
                .collect();
            list.insert(
                0,
                vec![
                    "Date".to_string(),
                    "Day".to_string(),
                    "Duration (hours)".to_string(),
                    "Customer".to_string(),
                    "Task".to_string(),
                    "Description".to_string(),
                ],
            );

            list.push(vec![
                "-".to_string(),
                "-".to_string(),
                activities
                    .iter()
                    .fold(0.0, |hours, activity| activity.hours + hours)
                    .to_string(),
                "-".to_string(),
                "-".to_string(),
                "".to_string(),
            ]);

            render_table(list);
        }
        cli::Commands::New {
            project,
            task,
            hours,
            date,
        } => {
            let (project, task) = prompt_task_select(&moco_client, project, task).await?;

            let date = if let Some(d) = date {
                d
            } else {
                print!("Date (YYYY-MM-DD) - Default 'today': ");
                std::io::stdout().flush()?;

                utils::read_line_date()
            };

            let hours = if let Some(h) = hours {
                h
            } else {
                let answer =
                    ask_question("Duration (hours) - Default 'start timer': ", &|answer| {
                        answer.is_empty().then_some(None).unwrap_or_else(|| {
                            answer.parse::<f64>().err().map(|e| format!("{}", e))
                        })
                    })?;
                answer
                    .is_empty()
                    .then_some(0_f64)
                    .unwrap_or_else(|| answer.parse::<f64>().unwrap())
            };

            moco_client
                .create_activity(&CreateActivity {
                    date: date.format(FORMAT_DATE).to_string(),
                    project_id: project.id,
                    task_id: task.id,
                    hours: Some(hours),
                    ..Default::default()
                })
                .await?;
        }
        cli::Commands::Edit { activity } => {
            let activity = prompt_activity_select(&moco_client, activity).await?;

            let now = Utc::now().format(FORMAT_DATE).to_string();

            print!("New date (YYYY-MM-DD) - Default '{}': ", activity.date);
            std::io::stdout().flush()?;

            let mut date = utils::read_line()?;
            if date.is_empty() {
                date = now.clone()
            }

            print!("New duration (hours) - Default '{}': ", activity.hours);
            std::io::stdout().flush()?;

            let mut hours = utils::read_line()?;
            if hours.is_empty() {
                hours = activity.hours.to_string()
            }

            print!("New description - Default 'current': ");
            std::io::stdout().flush()?;

            let mut description = utils::read_line()?;
            if description.is_empty() {
                description = activity
                    .description
                    .as_ref()
                    .unwrap_or(&String::new())
                    .to_string()
            }

            moco_client
                .edit_activity(&EditActivity {
                    activity_id: activity.id,
                    project_id: activity.project.id,
                    task_id: activity.task.id,
                    date,
                    description,
                    hours,
                })
                .await?;
        }
        cli::Commands::EditSimple { activity } => {
            let activity = prompt_activity_select_today(&moco_client, activity).await?;

            print!("New duration (hours) - Default '{}': ", activity.hours);
            std::io::stdout().flush()?;

            let mut hours = utils::read_line()?;
            if hours.is_empty() {
                hours = activity.hours.to_string()
            }

            moco_client
                .edit_activity(&EditActivity {
                    activity_id: activity.id,
                    project_id: activity.project.id,
                    task_id: activity.task.id,
                    date: activity.date,
                    description: activity.description.unwrap(),
                    hours,
                })
                .await?;
        }
        cli::Commands::Rm { activity, date } => {
            let activity = match date {
                Some(date) => {
                    println!("Delete activities for {}", date.format(FORMAT_DATE_DAY));
                    let date_string = date.format(FORMAT_DATE).to_string();
                    prompt_activity_select_date(
                        &moco_client,
                        activity,
                        date_string.clone(),
                        date_string,
                    )
                    .await
                }
                None => prompt_activity_select(&moco_client, activity).await,
            }?;

            moco_client
                .delete_activity(&DeleteActivity {
                    activity_id: activity.id,
                })
                .await?;
        }
        cli::Commands::Timer { system, activity } => match system {
            cli::Timer::Start => {
                let activity = prompt_activity_select_today(&moco_client, activity).await?;

                moco_client
                    .control_activity_timer(&ControlActivityTimer {
                        control: "start".to_string(),
                        activity_id: activity.id,
                    })
                    .await?;
            }
            cli::Timer::Stop => {
                let now = Utc::now().format(FORMAT_DATE).to_string();
                let from = now.clone();
                let to = now.clone();

                let activities = moco_client.get_activities(from, to, None, None).await?;
                let activity = activities.iter().find(|a| !a.timer_started_at.is_null());

                if let Some(a) = activity {
                    moco_client
                        .control_activity_timer(&ControlActivityTimer {
                            control: "stop".to_string(),
                            activity_id: a.id,
                        })
                        .await?;

                    let a = moco_client
                        .get_activity(&GetActivity { activity_id: a.id })
                        .await?;
                    println!("Activity duration: {} hours", a.hours);
                } else {
                    println!("Could not stop timer since it was not on");
                }
            }
        },
        cli::Commands::Overtime { monthly } => {
            let overtime = moco_client.get_user_performance_report().await?;

            if monthly {
                println!(
                    "Your monthly overtime report for {}",
                    overtime.annually.year
                );

                let mut list: Vec<Vec<String>> = overtime
                    .monthly
                    .iter()
                    .map(|month| {
                        vec![
                            format!("{:0>2}", month.month.to_string())
                                + ": "
                                + Month::from_i64(month.month).unwrap().name(),
                            month.variation.to_string(),
                            month.target_hours.to_string(),
                            month.hours_tracked_total.to_string(),
                        ]
                    })
                    .collect();

                list.insert(
                    0,
                    vec![
                        "Month".to_string(),
                        "Overtime".to_string(),
                        "Target Hours".to_string(),
                        "Tracked Hours".to_string(),
                    ],
                );

                list.push(vec![
                    "-------------".to_string(),
                    "--------".to_string(),
                    "------------".to_string(),
                    "-------------".to_string(),
                ]);

                list.push(vec![
                    "==>".to_string(),
                    overtime.annually.variation.to_string(),
                    overtime.annually.target_hours.to_string(),
                    overtime.annually.hours_tracked_total.to_string(),
                ]);

                render_table(list);
            } else {
                println!(
                    "Your current overtime until end of today: {}",
                    overtime.annually.variation_until_today
                );
            }
        }
    }

    Ok(())
}
