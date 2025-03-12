// fn history_title_row<'a>(
//     date: &NaiveDate,
//     total_time: i64,
//     total_earnings: f32,
//     settings: &FurSettings,
//     running_timer: Option<(bool, &str)>,
//     localization: &Localization,
// ) -> Row<'a, Message> {
//     let mut total_time_column = column![].align_x(Alignment::End);

//     if settings.show_daily_time_total {
//         if settings.dynamic_total {
//             if let Some((running, timer_text)) = running_timer {
//                 if running {
//                     let total_time_str = seconds_to_formatted_duration(
//                         combine_timer_with_seconds(timer_text, total_time),
//                         settings.show_seconds,
//                     );
//                     total_time_column =
//                         total_time_column.push(text(total_time_str).font(font::Font {
//                             weight: iced::font::Weight::Bold,
//                             ..Default::default()
//                         }));
//                 } else {
//                     let total_time_str =
//                         seconds_to_formatted_duration(total_time, settings.show_seconds);
//                     total_time_column =
//                         total_time_column.push(text(total_time_str).font(font::Font {
//                             weight: iced::font::Weight::Bold,
//                             ..Default::default()
//                         }));
//                 }
//             } else {
//                 let total_time_str =
//                     seconds_to_formatted_duration(total_time, settings.show_seconds);
//                 total_time_column = total_time_column.push(text(total_time_str).font(font::Font {
//                     weight: iced::font::Weight::Bold,
//                     ..Default::default()
//                 }));
//             }
//         } else {
//             let total_time_str = seconds_to_formatted_duration(total_time, settings.show_seconds);
//             total_time_column = total_time_column.push(text(total_time_str).font(font::Font {
//                 weight: iced::font::Weight::Bold,
//                 ..Default::default()
//             }));
//         }
//     }

//     if settings.show_task_earnings && total_earnings > 0.0 {
//         total_time_column = total_time_column.push(text!("${:.2}", total_earnings));
//     }

//     row![
//         text(format_history_date(date, localization)).font(font::Font {
//             weight: iced::font::Weight::Bold,
//             ..Default::default()
//         }),
//         horizontal_space().width(Length::Fill),
//         total_time_column,
//     ]
//     .align_y(Alignment::Center)
// }
