use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: i32,
    pub name: String,
    pub budget: Option<f64>,
    pub client_id: Option<i32>, // Foreign key to client table (optional)
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>, // Optional end date for the project
    pub category: Option<String>,  // Project category (e.g., web development, design)
    pub description: Option<String>, // More detailed project description
    pub files: Option<Vec<String>>,  // List of file paths related to the project
    pub client_name: Option<String>, // Client name (optional)
    pub priority: Option<i32>,       // Project priority level (e.g., 1 for high)
    pub communication_channels: Option<Vec<String>>, // Communication channels (e.g., email, Slack)
    pub milestones: Option<Vec<Milestone>>, // List of associated milestones (optional)
    pub enable_milestones: bool, // Flag indicating if milestone-based payments are enabled
}

impl Project {
    pub fn create(conn: &PgConnection, name: &str, budget: Option<f64>, client_id: Option<i32>) -> Result<Project, Box<dyn std::error::Error>> {
        let new_project = Project {
            id: 0, // Database will assign ID
            name: name.to_string(),
            budget,
            client_id,
            start_date: None,
            end_date: None,
            category: None,
            description: None,
            files: None,
            client_name: None,
            priority: None,
            communication_channels: None,
            milestones: None,
            enable_milestones: false, // Default for new projects (optional)
        };

        // Insert the project into the database using Diesel
        let inserted_project = diesel::insert_into(crate::schema::projects::table)
            .values(&new_project)
            .execute(&mut conn)?;

        Ok(inserted_project)
    }

    pub fn update(self, conn: &PgConnection) -> Result<Project, Box<dyn std::error::Error>> {
        diesel::update(crate::schema::projects::table)
            .filter(crate::schema::projects::id.eq(self.id))
            .set(&self)
            .execute(&mut conn)
            .map_err(|err| format!("Error updating project: {}", err))?;

        Ok(self)
    }

    pub fn get_by_id(id: i32, conn: &PgConnection) -> Result<Option<Project>, Box<dyn std::error::Error>> {
        Ok(crate::schema::projects::table.find(id).get_result::<Project>(conn).optional()?)
    }

    pub fn get_all(conn: &PgConnection) -> Result<Vec<Project>, Box<dyn std::error::Error>> {
        Ok(crate::schema::projects::table.load::<Project>(conn)?)
    }

    pub fn add_milestone(self, conn: &PgConnection, milestone: &Milestone) -> Result<Project, Box<dyn std::error::Error>> {
        let mut updated_project = self;
        if updated_project.milestones.is_none() {
            updated_project.milestones = Some(vec![]);
        }
        updated_project.milestones.as_mut().unwrap().push(milestone.clone());

        // Update the project in the database to reflect the new milestone
        updated_project.update(conn)?;

        Ok(updated_project)
    }

    pub fn get_milestones(self, conn: &PgConnection) -> Result<Vec<Milestone>, Box<dyn std::error::Error>> {
        // Assuming milestones is an Option<Vec<Milestone>>
        match self.milestones {
            Some(milestones) => Ok(milestones),
            None => Ok(vec![]), // Return an empty vector if no milestones are associated
        }
    }
    
    pub fn remove_milestone(self, conn: &PgConnection, milestone_id: i32) -> Result<Project, Box<dyn std::error::Error>> {
        let mut updated_project = self;
        if updated_project.milestones.is_none() {
            return Err(Box::new(diesel::result::Error::NotFound)); // No milestones to remove
        }
    
        let mut project_milestones = updated_project.milestones.unwrap();
        let milestone_index = project_milestones.iter().position(|m| m.id == milestone_id);
    
        match milestone_index {
            Some(index) => {
                project_milestones.remove(index);
                updated_project.milestones = Some(project_milestones);
    
                // Update the project in the database to reflect the removed milestone
                updated_project.update(conn)?;
            }
            None => return Err(Box::new(diesel::result::Error::NotFound)), // Milestone not found
        }
    
        Ok(updated_project)
    }
}
    
pub fn set_start_date(self, conn: &PgConnection, start_date: DateTime<Utc>) -> Result<Project, Box<dyn std::error::Error>> {
    // Update the start_date field and call update(conn)
}

pub fn set_end_date(self, conn: &PgConnection, end_date: DateTime<Utc>) -> Result<Project, Box<dyn std::error::Error>> {
    // Update the end_date field and call update(conn)
}

pub fn update_details(self, conn: &PgConnection, 
                      category: Option<String>, 
                      description: Option<String>, 
                      files: Option<Vec<String>>) -> Result<Project, Box<dyn std::error::Error>> {
    // Update relevant fields (category, description, files) and call update(conn)
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ProjectStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    // Existing fields...
    pub status: ProjectStatus,
    pub dependent_projects: Vec<i32>, // List of project IDs that this project depends on
}

impl Project {
    // Existing methods...

    pub fn set_status(&mut self, status: ProjectStatus) -> Result<(), Box<dyn std::error::Error>> {
        self.status = status;
        self.update(conn)?;
        Ok(())
    }

    pub fn add_dependency(&mut self, project_id: i32) -> Result<(), Box<dyn std::error::Error>> {
        self.dependent_projects.push(project_id);
        self.update(conn)?;
        Ok(())
    }

    pub fn remove_dependency(&mut self, project_id: i32) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(index) = self.dependent_projects.iter().position(|id| *id == project_id) {
            self.dependent_projects.remove(index);
            self.update(conn)?;
        }
        Ok(())
    }
}
