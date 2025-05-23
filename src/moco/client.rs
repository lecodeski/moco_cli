use std::{cell::RefCell, error::Error, sync::Arc};

use reqwest::Client;

use crate::moco::model::{
    Activity, ControlActivityTimer, CreateActivity, DeleteActivity, EditActivity, Employment,
    GetActivity, PerformanceReport, Projects,
};

use crate::config::AppConfig;

pub struct MocoClient {
    client: Client,
    config: Arc<RefCell<AppConfig>>,
}

#[derive(Debug, derive_more::Display)]
enum MocoClientError {
    NotLoggedIn,
}
impl Error for MocoClientError {}

#[allow(clippy::await_holding_refcell_ref)]
impl MocoClient {
    pub fn new(app_config: &Arc<RefCell<AppConfig>>) -> Self {
        MocoClient {
            client: Client::new(),
            config: app_config.clone(),
        }
    }

    pub async fn get_user_id(
        &self,
        firstname: String,
        lastname: String,
    ) -> Result<Option<i64>, Box<dyn Error>> {
        let config = &self.config.borrow();
        match (config.moco_api_key.as_ref(), config.moco_company.as_ref()) {
            (Some(api_key), Some(company)) => {
                let employments = self
                    .client
                    .get(format!(
                        "https://{company}.mocoapp.com/api/v1/users/employments"
                    ))
                    .header("Authorization", format!("Token token={}", api_key))
                    .send()
                    .await?
                    .json::<Vec<Employment>>()
                    .await?;
                Ok(employments
                    .iter()
                    .find(|employment| {
                        employment.user.firstname == firstname
                            && employment.user.lastname == lastname
                    })
                    .map(|employment| employment.user.id))
            }
            (_, _) => Err(Box::new(MocoClientError::NotLoggedIn)),
        }
    }

    pub async fn get_activities(
        &self,
        from: String,
        to: String,
        task_id: Option<String>,
        term: Option<String>,
    ) -> Result<Vec<Activity>, Box<dyn Error>> {
        let mut parameter = vec![
            ("from", from),
            ("to", to),
            (
                "user_id",
                format!("{}", &self.config.borrow().moco_user_id.unwrap()),
            ),
        ];

        if let Some(x) = task_id {
            parameter.push(("task_id", x))
        }
        if let Some(x) = term {
            parameter.push(("term", x))
        }

        let config = &self.config.borrow();
        match (config.moco_api_key.as_ref(), config.moco_company.as_ref()) {
            (Some(api_key), Some(company)) => Ok(self
                .client
                .get(format!("https://{company}.mocoapp.com/api/v1/activities"))
                .query(&parameter)
                .header("Authorization", format!("Token token={}", api_key))
                .send()
                .await?
                .json::<Vec<Activity>>()
                .await?),
            (_, _) => Err(Box::new(MocoClientError::NotLoggedIn)),
        }
    }

    pub async fn get_activity(&self, payload: &GetActivity) -> Result<Activity, Box<dyn Error>> {
        let config = &self.config.borrow();
        match (config.moco_api_key.as_ref(), config.moco_company.as_ref()) {
            (Some(api_key), Some(company)) => Ok(self
                .client
                .get(format!(
                    "https://{company}.mocoapp.com/api/v1/activities/{}",
                    payload.activity_id
                ))
                .header("Authorization", format!("Token token={}", api_key))
                .send()
                .await?
                .json::<Activity>()
                .await?),
            (_, _) => Err(Box::new(MocoClientError::NotLoggedIn)),
        }
    }

    pub async fn create_activity(&self, payload: &CreateActivity) -> Result<(), Box<dyn Error>> {
        let config = &self.config.borrow();
        match (config.moco_api_key.as_ref(), config.moco_company.as_ref()) {
            (Some(api_key), Some(company)) => {
                self.client
                    .post(format!("https://{company}.mocoapp.com/api/v1/activities"))
                    .header("Authorization", format!("Token token={}", api_key))
                    .json(payload)
                    .send()
                    .await?;
                Ok(())
            }
            (_, _) => Err(Box::new(MocoClientError::NotLoggedIn)),
        }
    }

    pub async fn edit_activity(&self, payload: &EditActivity) -> Result<(), Box<dyn Error>> {
        let config = &self.config.borrow();
        match (config.moco_api_key.as_ref(), config.moco_company.as_ref()) {
            (Some(api_key), Some(company)) => {
                self.client
                    .put(format!(
                        "https://{company}.mocoapp.com/api/v1/activities/{}",
                        payload.activity_id
                    ))
                    .header("Authorization", format!("Token token={}", api_key))
                    .json(payload)
                    .send()
                    .await?;
                Ok(())
            }
            (_, _) => Err(Box::new(MocoClientError::NotLoggedIn)),
        }
    }

    pub async fn delete_activity(&self, payload: &DeleteActivity) -> Result<(), Box<dyn Error>> {
        let config = &self.config.borrow();
        match (config.moco_api_key.as_ref(), config.moco_company.as_ref()) {
            (Some(api_key), Some(company)) => {
                self.client
                    .delete(format!(
                        "https://{company}.mocoapp.com/api/v1/activities/{}",
                        payload.activity_id
                    ))
                    .header("Authorization", format!("Token token={}", api_key))
                    .send()
                    .await?;
                Ok(())
            }
            (_, _) => Err(Box::new(MocoClientError::NotLoggedIn)),
        }
    }

    pub async fn control_activity_timer(
        &self,
        payload: &ControlActivityTimer,
    ) -> Result<(), Box<dyn Error>> {
        let config = &self.config.borrow();
        match (config.moco_api_key.as_ref(), config.moco_company.as_ref()) {
            (Some(api_key), Some(company)) => {
                self.client
                    .patch(format!(
                        "https://{company}.mocoapp.com/api/v1/activities/{}/{}_timer",
                        payload.activity_id, payload.control
                    ))
                    .header("Authorization", format!("Token token={}", api_key))
                    .send()
                    .await?;
                Ok(())
            }
            (_, _) => Err(Box::new(MocoClientError::NotLoggedIn)),
        }
    }

    pub async fn get_assigned_projects(&self) -> Result<Projects, Box<dyn Error>> {
        let config = &self.config.borrow();
        match (config.moco_api_key.as_ref(), config.moco_company.as_ref()) {
            (Some(api_key), Some(company)) => Ok(self
                .client
                .get(format!(
                    "https://{company}.mocoapp.com/api/v1/projects/assigned?active=true"
                ))
                .header("Authorization", format!("Token token={}", api_key))
                .send()
                .await?
                .json::<Projects>()
                .await?),
            (_, _) => Err(Box::new(MocoClientError::NotLoggedIn)),
        }
    }

    pub async fn get_user_performance_report(&self) -> Result<PerformanceReport, Box<dyn Error>> {
        let config = &self.config.borrow();
        match (
            config.moco_bot_api_key.as_ref(),
            config.moco_company.as_ref(),
            config.moco_user_id.as_ref(),
        ) {
            (Some(bot_api_key), Some(company), Some(user_id)) => Ok(self
                .client
                .get(format!(
                    "https://{company}.mocoapp.com/api/v1/users/{user_id}/performance_report"
                ))
                .header("Authorization", format!("Token token={}", bot_api_key))
                .send()
                .await?
                .json::<PerformanceReport>()
                .await?),
            (_, _, _) => Err(Box::new(MocoClientError::NotLoggedIn)),
        }
    }
}
