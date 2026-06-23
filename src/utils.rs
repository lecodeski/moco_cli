use crate::moco::client::MocoClient;
use crate::moco::model::{Activity, DeleteActivity, Project, ProjectTask};
//noinspection RsUnresolvedPath
use owo_colors::OwoColorize;
use std::{error::Error, io::Write, vec};

use chrono::Weekday::Mon;
use chrono::{Datelike, Duration, Local, Months, NaiveDate};
use now::DateTimeNow;
use tabled::builder::Builder;
use tabled::settings::object::Rows;
use tabled::settings::style::{BorderColor, HorizontalLine};
use tabled::settings::{Color, Style};
use unicode_ellipsis::truncate_str;

pub fn read_line() -> Result<String, Box<dyn Error>> {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    input.remove(input.len() - 1);
    Ok(input)
}

pub fn render_table(list: Vec<Vec<String>>) {
    if list.is_empty() {
        return;
    }

    let mut builder = Builder::default();
    for (row_index, row) in list.iter().enumerate() {
        let is_emphasized = row_index == 0 || row.first().map(|c| c == "==>").unwrap_or(false);
        let styled = row.iter().map(|cell| {
            if is_emphasized {
                cell.bold().to_string()
            } else if row_index % 2 == 0 {
                cell.to_string()
            } else {
                cell.green().to_string()
            }
        });
        builder.push_record(styled);
    }

    let mut table = builder.build();
    let line = HorizontalLine::new('─').intersection('+');

    let appendix = match list.last().unwrap().first().unwrap().as_str() {
        "==>" => 1,
        _ => 0,
    };
    table.with(Style::psql().horizontals([(1, line), (list.len() - appendix, line)]));
    table.modify(
        Rows::one(list.len() - appendix),
        BorderColor::new().top(Color::FG_GREEN),
    );
    println!("{}", table);
}

pub fn render_list_select<T>(
    list: &[T],
    headline: Vec<&str>,
    prompt: &str,
    linenderer: &dyn Fn((usize, &T)) -> Vec<String>,
) -> Result<usize, Box<dyn Error>> {
    loop {
        let mut rendered_list: Vec<Vec<String>> = list.iter().enumerate().map(linenderer).collect();
        rendered_list.insert(0, headline.iter().map(|x| x.to_string()).collect());
        render_table(rendered_list);

        print!("{}", prompt);
        std::io::stdout().flush()?;

        let index_input = read_line().map(|x| x.parse::<usize>().ok()).ok().flatten();

        if let Some(index) = index_input
            && index < list.len()
        {
            return Ok(index);
        }
        println!("Index Invalid")
    }
}

pub enum ListSelection {
    Index(usize),
    All,
}

pub fn render_list_select_all<T>(
    list: &[T],
    headline: Vec<&str>,
    prompt: &str,
    linenderer: &dyn Fn((usize, &T)) -> Vec<String>,
) -> Result<ListSelection, Box<dyn Error>> {
    loop {
        let mut rendered_list: Vec<Vec<String>> = list.iter().enumerate().map(linenderer).collect();
        rendered_list.insert(0, headline.iter().map(|x| x.to_string()).collect());
        render_table(rendered_list);

        print!("{}", prompt);
        std::io::stdout().flush()?;

        let input = read_line()?;

        if input.trim() == "A" {
            return Ok(ListSelection::All);
        }

        if let Some(index) = input.trim().parse::<usize>().ok()
            && index < list.len()
        {
            return Ok(ListSelection::Index(index));
        }
        println!("Index Invalid")
    }
}

pub fn select_from_to_date(
    day: Option<u32>,
    week: Option<u32>,
    month: Option<u32>,
    backward: bool,
) -> (NaiveDate, NaiveDate) {
    let now = Local::now();

    if let Some(day) = day {
        let target_day = if backward {
            match day {
                0 => print!("today, "),
                1 => print!("yesterday, "),
                _ => print!("{} days ago, ", day),
            }

            now.checked_sub_signed(Duration::days(day as i64))
                .unwrap()
                .date_naive()
        } else {
            print!("Day {} in {}, ", day, now.year());
            NaiveDate::from_yo_opt(now.year(), day).expect("invalid day of year")
        };
        (target_day, target_day)
    } else if let Some(week) = week {
        let target_week = if backward {
            let then = now
                .checked_sub_signed(Duration::weeks(week as i64))
                .unwrap()
                .date_naive();

            let iso_week = then.iso_week().week();
            match week {
                0 => print!("current week (CW {}), ", iso_week),
                1 => print!("last week (CW {}), ", iso_week),
                _ => print!("CW {} ({} weeks ago), ", iso_week, week),
            }

            then
        } else {
            print!("CW {}, ", week);
            NaiveDate::from_isoywd_opt(now.year(), week, Mon).expect("invalid calendar week")
        }
        .week(Mon);
        (target_week.first_day(), target_week.last_day())
    } else if let Some(month) = month {
        let target_month = if backward {
            let then = now.checked_sub_months(Months::new(month)).unwrap();

            let then_month = then.format("%B");
            match month {
                0 => print!("current month ({}), ", then_month),
                1 => print!("last month ({}), ", then_month),
                _ => print!("{} ({} months ago), ", then_month, month),
            }

            then
        } else {
            let then = now.with_month(month).unwrap();
            print!("{}, ", then.format("%B"));
            then
        };

        (
            target_month.beginning_of_month().date_naive(),
            target_month.end_of_month().date_naive(),
        )
    } else {
        (now.date_naive(), now.date_naive())
    }
}

pub fn ask_question_mandatory(
    question: &str,
    validator: &dyn Fn(&str) -> Option<String>,
) -> Result<String, Box<dyn Error>> {
    loop {
        print!("{}", question);
        std::io::stdout().flush()?;
        let line = read_line()?;
        if let Some(error) = validator(&line) {
            println!("{}", error);
            continue;
        }
        return Ok(line);
    }
}

pub fn ask_question<T>(
    question: &str,
    validator: &dyn Fn(&str) -> Result<T, Box<dyn Error>>,
) -> Result<T, Box<dyn Error>> {
    loop {
        print!("{}", question);
        std::io::stdout().flush()?;
        let line = read_line()?;
        let result = validator(&line);
        if let Err(error) = result {
            println!("{}", error);
            continue;
        }
        return result;
    }
}

pub fn mandatory_validator(input: &str) -> Option<String> {
    if input.is_empty() {
        Some("Input is required".to_string())
    } else {
        None
    }
}

pub async fn prompt_task_select(
    moco_client: &MocoClient,
    project: Option<i64>,
    task_id: Option<i64>,
) -> Result<(Project, ProjectTask), Box<dyn Error>> {
    let projects = moco_client.get_assigned_projects().await?;
    let project = projects.iter().find(|p| p.id == project.unwrap_or(-1));

    let project = if let Some(p) = project {
        p
    } else {
        let project_index = render_list_select(
            &projects,
            vec!["Index", "Customer", "Project", "Project ID"],
            "Choose your Project: ",
            &(|(index, project)| {
                vec![
                    index.to_string(),
                    project.customer.name.clone(),
                    project.name.clone(),
                    project.id.to_string(),
                ]
            }),
        )?;
        println!();

        &projects[project_index]
    };

    let active_tasks: Vec<&ProjectTask> = project.tasks.iter().filter(|t| t.active).collect();
    let selected_task = active_tasks.iter().find(|t| t.id == task_id.unwrap_or(-1));

    let task = if let Some(t) = selected_task {
        t
    } else {
        let task_index = render_list_select(
            &active_tasks,
            vec!["Index", "Task", "Task ID"],
            "Choose your Task: ",
            &(|(index, task)| vec![index.to_string(), task.name.clone(), task.id.to_string()]),
        )?;
        active_tasks[task_index]
    };

    Ok((project.clone(), task.clone()))
}

pub async fn activity_select(
    moco_client: &MocoClient,
    activity: Option<i64>,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Activity, Box<dyn Error>> {
    let activities = moco_client.get_activities(from, to, None, None).await?;
    let activity = activities.iter().find(|a| a.id == activity.unwrap_or(-1));

    let activity = if let Some(a) = activity {
        a
    } else {
        let activity_index = render_list_select(
            &activities,
            vec![
                "Index",
                "Date",
                "Hours",
                "Customer",
                "Project",
                "Task",
                "Description",
            ],
            "Choose your Activity: ",
            &(|(index, activity)| {
                vec![
                    index.to_string(),
                    activity.date.clone(),
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
            }),
        )?;

        &activities[activity_index]
    };

    Ok(activity.clone())
}

pub fn prompt_from_to_date() -> Result<(NaiveDate, NaiveDate), Box<dyn Error>> {
    let now = Local::now().date_naive();

    print!("List activities from (YYYY-MM-DD) - Default 'today': ");
    std::io::stdout().flush()?;

    let from_input = read_line()?;
    let from = if from_input.is_empty() {
        now
    } else {
        from_input.parse::<NaiveDate>()?
    };

    print!("List activities to (YYYY-MM-DD) - Default 'last answer': ");
    std::io::stdout().flush()?;

    let to_input = read_line()?;
    let to = if to_input.is_empty() {
        from
    } else {
        to_input.parse::<NaiveDate>()?
    };

    Ok((from, to))
}

pub async fn activity_delete_loop(
    moco_client: &MocoClient,
    mut activity: Option<i64>,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<(), Box<dyn Error>> {
    loop {
        let activities = moco_client.get_activities(from, to, None, None).await?;

        if activities.is_empty() {
            println!("No (more) activities to delete");
            break;
        }

        // If an activity id was passed directly, delete it once and keep looping.
        if let Some(id) = activity.take()
            && let Some(a) = activities.iter().find(|a| a.id == id)
        {
            moco_client
                .delete_activity(&DeleteActivity { activity_id: a.id })
                .await?;
            continue;
        }

        let selection = render_list_select_all(
            &activities,
            vec![
                "Index",
                "Date",
                "Hours",
                "Customer",
                "Project",
                "Task",
                "Description",
            ],
            "Choose your Activity ('A' deletes all): ",
            &(|(index, activity)| {
                vec![
                    index.to_string(),
                    activity.date.clone(),
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
            }),
        )?;

        match selection {
            ListSelection::All => {
                for a in &activities {
                    moco_client
                        .delete_activity(&DeleteActivity { activity_id: a.id })
                        .await?;
                }
                break;
            }
            ListSelection::Index(index) => {
                moco_client
                    .delete_activity(&DeleteActivity {
                        activity_id: activities[index].id,
                    })
                    .await?;
            }
        }
    }

    Ok(())
}

pub async fn prompt_activity_select_today(
    moco_client: &MocoClient,
    activity: Option<i64>,
) -> Result<Activity, Box<dyn Error>> {
    let now = Local::now().date_naive();

    println!("List activities for today: ");

    activity_select(moco_client, activity, now, now).await
}
