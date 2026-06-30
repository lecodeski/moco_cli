use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

// Employment

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Employment {
    pub(crate) id: i64,
    #[serde(rename = "weekly_target_hours")]
    pub(crate) weekly_target_hours: f64,
    pub(crate) pattern: Pattern,
    pub(crate) from: String,
    pub(crate) to: Value,
    pub(crate) user: User,
    #[serde(rename = "created_at")]
    pub(crate) created_at: String,
    #[serde(rename = "updated_at")]
    pub(crate) updated_at: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Pattern {
    pub(crate) am: Vec<f64>,
    pub(crate) pm: Vec<f64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct User {
    pub(crate) id: i64,
    pub(crate) firstname: String,
    pub(crate) lastname: String,
}

// Activity

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Activity {
    pub(crate) id: i64,
    pub(crate) date: String,
    pub(crate) hours: f64,
    pub(crate) seconds: i64,
    pub(crate) description: Option<String>,
    pub(crate) billed: bool,
    pub(crate) billable: bool,
    pub(crate) tag: String,
    #[serde(rename = "remote_service")]
    pub(crate) remote_service: Option<String>,
    #[serde(rename = "remote_id")]
    pub(crate) remote_id: Option<String>,
    #[serde(rename = "remote_url")]
    pub(crate) remote_url: Value,
    pub(crate) project: ActivityProject,
    pub(crate) task: Task,
    pub(crate) customer: Customer,
    pub(crate) user: User,
    #[serde(rename = "timer_started_at")]
    pub(crate) timer_started_at: Value,
    #[serde(rename = "created_at")]
    pub(crate) created_at: String,
    #[serde(rename = "updated_at")]
    pub(crate) updated_at: String,
    #[serde(rename = "hourly_rate")]
    pub(crate) hourly_rate: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ActivityProject {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) billable: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Task {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) billable: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Customer {
    pub(crate) id: i64,
    pub(crate) name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetActivity {
    #[serde(rename = "activity_id")]
    pub(crate) activity_id: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateActivity {
    pub(crate) date: String,
    pub(crate) description: String,
    #[serde(rename = "project_id")]
    pub(crate) project_id: i64,
    #[serde(rename = "task_id")]
    pub(crate) task_id: i64,
    pub(crate) hours: Option<f64>,
    pub(crate) seconds: Option<i64>,
    pub(crate) tag: Option<String>,
    #[serde(rename = "remote_service")]
    pub(crate) remote_service: Option<String>,
    #[serde(rename = "remote_id")]
    pub(crate) remote_id: Option<String>,
    #[serde(rename = "remote_url")]
    pub(crate) remote_url: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EditActivity {
    #[serde(rename = "activity_id")]
    pub(crate) activity_id: i64,
    #[serde(rename = "project_id")]
    pub(crate) project_id: i64,
    #[serde(rename = "task_id")]
    pub(crate) task_id: i64,
    pub(crate) date: String,
    pub(crate) description: String,
    pub(crate) hours: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ControlActivityTimer {
    pub(crate) control: String,
    #[serde(rename = "activity_id")]
    pub(crate) activity_id: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DeleteActivity {
    #[serde(rename = "activity_id")]
    pub(crate) activity_id: i64,
}

//Project

pub(crate) type Projects = Vec<Project>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Project {
    pub(crate) id: i64,
    pub(crate) identifier: String,
    pub(crate) name: String,
    pub(crate) active: bool,
    pub(crate) billable: bool,
    pub(crate) customer: Customer,
    pub(crate) tasks: Vec<ProjectTask>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectTask {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) active: bool,
    pub(crate) billable: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct PerformanceReport {
    pub(crate) annually: PerformanceReportAnnually,
    pub(crate) monthly: Vec<PerformanceReportMonthly>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct PerformanceReportAnnually {
    pub(crate) year: i64,
    pub(crate) employment_hours: f64,
    pub(crate) target_hours: f64,
    pub(crate) hours_tracked_total: f64,
    pub(crate) variation: f64,
    pub(crate) variation_until_today: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct PerformanceReportMonthly {
    pub(crate) year: i64,
    pub(crate) month: u32,
    pub(crate) target_hours: f64,
    pub(crate) hours_tracked_total: f64,
    pub(crate) variation: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct WorkTimeAdjustment {
    pub(crate) date: String,
    pub(crate) hours: f64,
}
