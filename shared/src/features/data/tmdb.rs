use url::Url;

use crate::{
    Command, CruxContext, Model,
    capabilities::http,
    features::{query::QueryState, utils::update_model},
};

pub fn search_tmdb_media(model: &mut Model, query: String) -> Command {
    let page = match &model.tmdb_search_results {
        // New query
        None => 1,
        Some((existing_query, _)) if *existing_query != query => 1,

        // We previously failed
        Some((_, QueryState::Error { .. })) => 1,

        // Same query is called again
        Some((
            existing_query,
            QueryState::Success {
                data: existing_results,
            },
        )) if *existing_query == query => {
            // We haven't fetched all pages next
            if existing_results.page <= existing_results.total_pages {
                existing_results.page + 1
            } else {
                // All pages were already fetched but shell wanted to fetch the next page
                // TODO log
                return Command::done();
            }
        }

        // Shell wanted to fetch next page while there is an existing one
        Some((_, QueryState::Loading { .. })) => {
            // TODO log
            return Command::done();
        }

        _ => 1,
    };

    let existing_state = model.tmdb_search_results.clone();
    Command::new(async move |ctx| {
        // 2a. set the result to loading
        update_model(
            &ctx,
            crate::PartialModel {
                tmdb_search_results: Some(existing_state.clone()),
                ..Default::default()
            },
        );

        let new_state = match tmdb::multi_search(api_call, ctx.clone(), &query, page).await {
            Ok(tmdb::PaginatedMultiSearchResult {
                page,
                total_pages,
                total_results,
                results: new_results,
            }) => {
                let existing_results: Option<Vec<tmdb::MultiSearchResult>> = match existing_state {
                    Some((
                        _,
                        QueryState::Success {
                            data: tmdb::PaginatedMultiSearchResult { results, .. },
                        },
                    )) => Some(results),
                    _ => None,
                };

                let results = match existing_results {
                    Some(mut existing_results) => {
                        existing_results.extend(new_results);
                        existing_results
                    }
                    None => new_results,
                };

                (
                    query,
                    QueryState::Success {
                        data: tmdb::PaginatedMultiSearchResult {
                            page,
                            total_pages,
                            total_results,
                            results,
                        },
                    },
                )
            }
            Err(err) => (
                query,
                QueryState::Error {
                    message: err.to_string(),
                },
            ),
        };

        update_model(
            &ctx,
            crate::PartialModel {
                tmdb_search_results: Some(Some(new_state)),
                ..Default::default()
            },
        );
    })
}

async fn api_call(
    context: CruxContext,
    url: Url,
    headers: &[(&str, &str)],
) -> Result<String, String> {
    match http::get(url).into_future(context).await {
        http::HttpOutput::Success {
            data: Some(data), ..
        } => Ok(data),
        // TODO replace with actual error message (depends on capability)
        _ => Err("API Request Failed".to_string()),
    }
}
