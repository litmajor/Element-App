use crate::models::{FreelancerAccount, ClientReview, Location, ContactInformation};

// Function to create a freelancer account
pub async fn create_freelancer_account(
    user_id: i32,
    hourly_rate: Option<f64>,
    project_pricing: Option<String>,
    specializations: Vec<String>,
    experience: Option<i32>,
    client_reviews: Vec<ClientReview>,
    average_rating: Option<f64>,
    location: Option<Location>,
    contact_information: Option<ContactInformation>,
) -> Result<FreelancerAccount, Error> {
    // Perform validation if needed
    
    // Create a new freelancer account object
    let freelancer_account = FreelancerAccount {
        id: 0, // Assuming the ID will be assigned by the database
        user_id,
        hourly_rate,
        project_pricing,
        specializations,
        experience,
        client_reviews,
        average_rating,
        location,
        contact_information,
    };

    // Insert the freelancer account into the database
    let created_freelancer_account = create_freelancer_account_in_database(&freelancer_account).await?;

    Ok(created_freelancer_account)
}

// Function to create a client review
pub async fn create_client_review(
    freelancer_id: i32,
    client_name: String,
    feedback: String,
    rating: f64,
) -> Result<ClientReview, Error> {
    // Perform validation if needed
    
    // Create a new client review object
    let client_review = ClientReview {
        client_name,
        feedback,
        rating,
    };

    // Insert the client review into the database
    let created_client_review = create_client_review_in_database(freelancer_id, &client_review).await?;

    Ok(created_client_review)
}

