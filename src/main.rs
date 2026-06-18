use chrono::{Datelike, Local, Month, NaiveDate};
use num_traits::FromPrimitive;
use owo_colors::OwoColorize;
use std::rc::Rc;
use std::{cell::RefCell, error::Error, io::Write, vec};
use unicode_ellipsis::truncate_str;
use utils::{prompt_activity_select, prompt_task_select, render_table};

use crate::moco::model::{ControlActivityTimer, CreateActivity, DeleteActivity, GetActivity};
use crate::utils::{activity_select, prompt_activity_select_today};
use crate::{
    moco::{client::MocoClient, model::EditActivity},
    utils::{ask_question, mandatory_validator},
};

mod cli;
mod config;
mod moco;

mod utils;

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
    let config = Rc::new(RefCell::new(config::init()?));
    let moco_client = MocoClient::new(&config);

    match args.command {
        cli::Commands::Login => {
            println!("MOCO Login");

            let moco_company = ask_question("Enter your company's name: ", &mandatory_validator)?;
            let api_key = ask_question("Enter your personal API key: ", &mandatory_validator)?;
            let bot_api_key = ask_question("Enter the MOCO Bot API key: ", &mandatory_validator)?;

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
        cli::Commands::List {
            week,
            month,
            backward,
            date,
        } => {
            print!("List activities for ");
            let activities = match date {
                Some(date) => {
                    println!("{}\n", date.format(FORMAT_DATE_DAY));
                    moco_client.get_activities(date, date, None, None)
                }
                None => {
                    let (from, to) = utils::select_from_to_date(week, month, backward);
                    if from == to {
                        println!("{}\n", from.format(FORMAT_DATE_DAY))
                    } else {
                        println!(
                            "from {} – {}\n",
                            from.format(FORMAT_DATE_DAY),
                            to.format(FORMAT_DATE_DAY)
                        )
                    };
                    moco_client.get_activities(from.date_naive(), to.date_naive(), None, None)
                }
            }
            .await?;

            let mut list: Vec<Vec<String>> = activities
                .iter()
                .map(|activity| {
                    vec![
                        activity.date.clone(),
                        activity
                            .date
                            .parse::<NaiveDate>()
                            .unwrap()
                            .weekday()
                            .to_string(),
                        activity.hours.to_string(),
                        truncate_str(&activity.customer.name, 16).to_string(),
                        truncate_str(&activity.project.name, 14).to_string(),
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
                    "Hours".to_string(),
                    "Customer".to_string(),
                    "Project".to_string(),
                    "Task".to_string(),
                    "Description".to_string(),
                ],
            );

            list.push(vec![
                "==>".to_string(),
                "".to_string(),
                activities
                    .iter()
                    .fold(0.0, |hours, activity| activity.hours + hours)
                    .to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            ]);

            render_table(list);
        }
        cli::Commands::New {
            project,
            task,
            hours,
            date,
            description,
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
                        if answer.is_empty() {
                            None
                        } else {
                            answer.parse::<f64>().err().map(|e| format!("{}", e))
                        }
                    })?;
                if answer.is_empty() {
                    0_f64
                } else {
                    answer.parse::<f64>()?
                }
            };

            let description = if let Some(d) = description {
                d
            } else {
                print!("Description: ");
                std::io::stdout().flush()?;
                utils::read_line()?
            };

            moco_client
                .create_activity(&CreateActivity {
                    date: date.to_string(),
                    project_id: project.id,
                    task_id: task.id,
                    hours: Some(hours),
                    description,
                    ..Default::default()
                })
                .await?;
        }
        cli::Commands::Edit { activity, date } => {
            let activity = match date {
                Some(date) => {
                    println!("Edit activities for {}", date.format(FORMAT_DATE_DAY));
                    activity_select(&moco_client, activity, date, date).await
                }
                None => prompt_activity_select(&moco_client, activity).await,
            }?;

            let now = Local::now().to_string();

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

            print!(
                "New description - Default '{}': ",
                activity.description.as_deref().unwrap_or("")
            );
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
        cli::Commands::Rm { activity, date } => {
            let activity = match date {
                Some(date) => {
                    println!("Delete activities for {}", date.format(FORMAT_DATE_DAY));
                    activity_select(&moco_client, activity, date, date).await
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
                let now = Local::now().date_naive();
                let activities = moco_client.get_activities(now, now, None, None).await?;
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
                    .map(|report| {
                        vec![
                            format!("{:0>2}", report.month.to_string())
                                + ": "
                                + Month::from_i64(report.month).unwrap().name(),
                            report.variation.to_string(),
                            report.target_hours.to_string(),
                            report.hours_tracked_total.to_string(),
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
                    "==>".to_string(),
                    overtime.annually.variation.to_string(),
                    overtime.annually.target_hours.to_string(),
                    overtime.annually.hours_tracked_total.to_string(),
                ]);

                render_table(list);
            } else {
                println!(
                    "Your current overtime until end of today: {}",
                    overtime.annually.variation_until_today.to_string().bold()
                );
            }
        }
    }

    Ok(())
}
