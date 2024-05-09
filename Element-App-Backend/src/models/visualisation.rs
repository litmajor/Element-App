use plotters::prelude::*;
use plotters::coord::{Shift, ShiftType};

impl InMemoryStore {
    // Function to generate budget utilization chart using plotters
    fn generate_budget_utilization_chart(&self, data: &Vec<(String, f64, f64)>, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(filename, (600, 400)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption("Budget Utilization", ("Arial", 16).into_font())
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_ranged(0..data.len() as i32, 0.0..100.0)?;

        chart
            .configure_mesh()
            .x_desc("Category")
            .y_desc("Amount")
            .disable_x_mesh()
            .disable_y_mesh()
            .draw()?;

        let mut bars = vec![];
        for (i, (category, income, expense)) in data.iter().enumerate() {
            bars.push(BarData::new(i as i32, *income, BLUE));
            bars.push(BarData::new(i as i32, *expense, RED));
        }

        chart.draw_series(
            bars.into_iter().map(|bar| {
                Rectangle::new([(bar.x - 0.4, 0.0), (bar.x + 0.4, bar.value)], bar.color.filled())
            })
        )?;

        Ok(())
    }
}
