

// pub fn score(issues: &Vec<crate::domain::analysis_result::Issue>) -> crate::domain::analysis_result::Scoring {
//     let total_issues = issues.len() as f32;
//     let risk_issues = issues.iter().filter(|issue| issue.category == crate::domain::analysis_result::Category::Risk).count() as f32;

//     let clarity = if total_issues == 0.0 {
//         1.0
//     } else {
//         1.0 - (issues.iter().filter(|issue| issue.category == crate::domain::analysis_result::Category::Ambiguity).count() as f32 / total_issues)
//     };
//     let completeness = if total_issues == 0.0 {
//         1.0
//     } else {
//         1.0 - (issues.iter().filter(|issue| issue.category == crate::domain::analysis_result::Category::MissingDetail).count() as f32 / total_issues)
//     };
//     let consistency = if total_issues == 0.0 {
//         1.0
//     } else {
//         1.0 - (issues.iter().filter(|issue| issue.category == crate::domain::analysis_result::Category::Contradiction).count() as f32 / total_issues)
//     };
//     let risk_level = if total_issues == 0.0 {
//         0.0
//     } else {
//         risk_issues / total_issues
//     };
//     let maturity = (clarity + completeness + consistency) / 3.0;

//     crate::domain::analysis_result::Scoring {
//         clarity,
//         completeness,
//         consistency,
//         risk_level,
//         maturity,
//     }
// }