use chrono::{Datelike, Local, Month, NaiveDate};
use num_traits::FromPrimitive;
//noinspection RsUnresolvedPath
use owo_colors::OwoColorize;
use std::rc::Rc;
use std::{cell::RefCell, error::Error, io::Write, vec};
use unicode_ellipsis::truncate_str;
use utils::{prompt_task_select, render_table};

use crate::moco::model::{ControlActivityTimer, CreateActivity, DeleteActivity, GetActivity};
use crate::utils::{
    activity_delete_loop, activity_select, ask_question_mandatory, footer,
    prompt_activity_select_today, prompt_from_to_date,
};
use crate::{
    moco::{client::MocoClient, model::EditActivity},
    utils::{ask_question, mandatory_validator},
};

mod cli;
mod config;
mod moco;

mod utils;

const FORMAT_DATE_DAY: &str = "%A %Y-%m-%d";
const FORMAT_DATE_DAY_WEEK: &str = constcat::concat!(FORMAT_DATE_DAY, " (CW %V)");

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

            let moco_company =
                ask_question_mandatory("Enter your company's name: ", &mandatory_validator)?;
            let api_key =
                ask_question_mandatory("Enter your personal API key: ", &mandatory_validator)?;
            let bot_api_key =
                ask_question_mandatory("Enter the MOCO Bot API key: ", &mandatory_validator)?;

            config.borrow_mut().moco_company = Some(moco_company);
            config.borrow_mut().moco_api_key = Some(api_key);
            config.borrow_mut().moco_bot_api_key = Some(bot_api_key);

            let firstname = ask_question_mandatory("Enter firstname: ", &mandatory_validator)?;
            let lastname = ask_question_mandatory("Enter lastname:  ", &mandatory_validator)?;

            let client_id = moco_client.get_user_id(firstname, lastname).await?;

            config.borrow_mut().moco_user_id = client_id;
            config.borrow_mut().write_config()?;
            println!("🤩 Logged in 🤩")
        }
        cli::Commands::List {
            day,
            week,
            month,
            backward,
            date,
        } => {
            print!("List activities for ");

            let (from, to) = match date {
                Some(date) => (date, date),
                None => utils::select_from_to_date(day, week, month, backward),
            };

            if from == to {
                println!("{}\n", from.format(FORMAT_DATE_DAY_WEEK))
            } else {
                println!(
                    "from {} – {}\n",
                    from.format(FORMAT_DATE_DAY),
                    to.format(FORMAT_DATE_DAY)
                )
            };

            let activities = moco_client.get_activities(from, to, None, None).await?;

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

            list.push(footer(activities.clone()));

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
                ask_question(
                    "Date (YYYY-MM-DD) - Default 'today': ",
                    &|answer| match answer.to_string() {
                        s if s.is_empty() => Ok(Local::now().date_naive()),
                        s => Ok(s.parse()?),
                    },
                )?
            };

            let hours = if let Some(h) = hours {
                h
            } else {
                ask_question("Duration (hours) - Default 'start timer': ", &|answer| {
                    Ok(match answer.replacen(',', ".", 1) {
                        s if s.is_empty() => "0".to_string(),
                        s if s == "." => "".to_string(),
                        s if s.starts_with('.') => format!("0{}", s),
                        s => s,
                    }
                    .parse::<f64>()?)
                })?
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
                    println!("Edit activities for {}", date.format(FORMAT_DATE_DAY_WEEK));
                    activity_select(&moco_client, activity, date, date).await
                }
                None => {
                    let (from, to) = prompt_from_to_date()?;
                    activity_select(&moco_client, activity, from, to).await
                }
            }?;

            let date = ask_question(
                &format!("New date (YYYY-MM-DD) - Default '{}': ", activity.date),
                &|answer| {
                    Ok(match answer.to_string() {
                        s if s.is_empty() => activity.date.clone(),
                        s => s,
                    }
                    .parse::<NaiveDate>()?)
                },
            )?;

            let hours = ask_question(
                &format!("New duration (hours) - Default '{}': ", activity.hours),
                &|answer| {
                    Ok(match answer.replacen(',', ".", 1) {
                        s if s.is_empty() => activity.hours.to_string(),
                        s if s == "." => "".to_string(),
                        s if s.starts_with('.') => format!("0{}", s),
                        s => s,
                    }
                    .parse::<f64>()?)
                },
            )?;

            let description = ask_question(
                &format!(
                    "New description - Default '{}': ",
                    activity.description.clone().unwrap()
                ),
                &|answer| {
                    Ok(match answer.to_string() {
                        s if s.is_empty() => activity.description.clone().unwrap(),
                        s => s,
                    })
                },
            )?;

            moco_client
                .edit_activity(&EditActivity {
                    activity_id: activity.id,
                    project_id: activity.project.id,
                    task_id: activity.task.id,
                    date: date.to_string(),
                    description,
                    hours: hours.to_string(),
                })
                .await?;
        }
        cli::Commands::Rm {
            activity,
            date,
            r#loop,
        } => {
            let (from, to) = match date {
                Some(date) => {
                    println!(
                        "Delete activities for {}",
                        date.format(FORMAT_DATE_DAY_WEEK)
                    );
                    (date, date)
                }
                None => prompt_from_to_date()?,
            };

            if r#loop {
                activity_delete_loop(&moco_client, activity, from, to).await?;
            } else {
                moco_client
                    .delete_activity(&DeleteActivity {
                        activity_id: activity_select(&moco_client, activity, from, to).await?.id,
                    })
                    .await?;
            }
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
