use actix_cors::Cors;
use actix_files::NamedFile;
use actix_web::{http::header, middleware, web, App, HttpServer};
use std::sync::Arc;
use stdx::log::info;

mod context;
mod middlewares;

pub mod api;
pub use context::{RequestContext, ServerContext};

async fn route_index() -> Result<NamedFile, actix_web::Error> {
    Ok(NamedFile::open("public/index.html")?)
}

pub async fn run(
    kernel_service: Arc<kernel::Service>,
    files_service: Arc<files::Service>,
    analytics_service: Arc<analytics::Service>,
    inbox_service: Arc<inbox::Service>,
) -> Result<(), ::kernel::Error> {
    let config = kernel_service.config();
    let context = Arc::new(ServerContext {
        kernel_service: kernel_service.clone(),
        files_service,
        analytics_service,
        inbox_service,
    });

    let endpoint = format!("0.0.0.0:{}", config.http.port);
    info!("Starting HTTP server. endpoint={:?}", &endpoint);
    HttpServer::new(move || {
        App::new()
            .data(Arc::clone(&context))
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                    .allowed_headers(vec![
                        header::AUTHORIZATION,
                        header::ACCEPT,
                        header::CONTENT_TYPE,
                        header::ORIGIN,
                    ])
                    .max_age(3600),
            )
            .wrap(middlewares::AuthMiddleware::new(kernel_service.clone()))
            .wrap(middleware::Compress::default())
            .wrap(middlewares::RequestIdMiddleware)
            .wrap(middlewares::SecurityHeadersMiddleware::new(Arc::clone(&config)))
            .wrap(middlewares::CacheHeadersMiddleware)
            .service(
                web::scope("/api")
                    .wrap(middlewares::NoCacheHeadersMiddleware)
                    .service(web::resource("").route(web::get().to(api::index)))
                    // kernel
                    .service(
                        web::scope("/kernel")
                            .service(
                                web::scope("/commands")
                                    // User
                                    .service(
                                        web::resource("/register")
                                            .route(web::post().to(api::kernel::commands::register)),
                                    )
                                    .service(
                                        web::resource("/complete_registration")
                                            .route(web::post().to(api::kernel::commands::complete_registration)),
                                    )
                                    .service(
                                        web::resource("/sign_in").route(web::post().to(api::kernel::commands::sign_in)),
                                    )
                                    .service(
                                        web::resource("/complete_sign_in")
                                            .route(web::post().to(api::kernel::commands::complete_sign_in)),
                                    )
                                    .service(
                                        web::resource("/revoke_session")
                                            .route(web::post().to(api::kernel::commands::revoke_session)),
                                    )
                                    .service(
                                        web::resource("/verify_email")
                                            .route(web::post().to(api::kernel::commands::verify_email)),
                                    )
                                    .service(
                                        web::resource("/delete_my_account")
                                            .route(web::post().to(api::kernel::commands::delete_my_account)),
                                    )
                                    .service(
                                        web::resource("/complete_two_fa_setup")
                                            .route(web::post().to(api::kernel::commands::complete_two_fa_setup)),
                                    )
                                    .service(
                                        web::resource("/setup_two_fa")
                                            .route(web::post().to(api::kernel::commands::setup_two_fa)),
                                    )
                                    .service(
                                        web::resource("/disable_two_fa")
                                            .route(web::post().to(api::kernel::commands::disable_two_fa)),
                                    )
                                    .service(
                                        web::resource("/complete_two_fa_challenge")
                                            .route(web::post().to(api::kernel::commands::complete_two_fa_challenge)),
                                    )
                                    .service(
                                        web::resource("/update_my_profile")
                                            .route(web::post().to(api::kernel::commands::update_my_profile)),
                                    )
                                    // Group
                                    .service(
                                        web::resource("/create_group")
                                            .route(web::post().to(api::kernel::commands::create_group)),
                                    )
                                    .service(
                                        web::resource("/delete_group")
                                            .route(web::post().to(api::kernel::commands::delete_group)),
                                    )
                                    .service(
                                        web::resource("/update_group_profile")
                                            .route(web::post().to(api::kernel::commands::update_group_profile)),
                                    )
                                    .service(
                                        web::resource("/quit_group")
                                            .route(web::post().to(api::kernel::commands::quit_group)),
                                    )
                                    .service(
                                        web::resource("/invite_people_in_group")
                                            .route(web::post().to(api::kernel::commands::invite_people_in_group)),
                                    )
                                    .service(
                                        web::resource("/accept_group_invitation")
                                            .route(web::post().to(api::kernel::commands::accept_group_invitation)),
                                    )
                                    .service(
                                        web::resource("/decline_group_invitation")
                                            .route(web::post().to(api::kernel::commands::decline_group_invitation)),
                                    )
                                    .service(
                                        web::resource("/cancel_group_invitation")
                                            .route(web::post().to(api::kernel::commands::cancel_group_invitation)),
                                    )
                                    .service(
                                        web::resource("/remove_member_from_group")
                                            .route(web::post().to(api::kernel::commands::remove_member_from_group)),
                                    )
                                    .service(
                                        web::resource("/update_billing_information")
                                            .route(web::post().to(api::kernel::commands::update_billing_information)),
                                    )
                                    .service(
                                        web::resource("/sync_customer_with_provider")
                                            .route(web::post().to(api::kernel::commands::sync_customer_with_provider)),
                                    ),
                            )
                            .service(
                                web::scope("/queries")
                                    .service(
                                        web::resource("/signed_upload_url")
                                            .route(web::post().to(api::kernel::queries::signed_upload_url)),
                                    )
                                    .service(web::resource("/me").route(web::post().to(api::kernel::queries::me)))
                                    .service(
                                        web::resource("/markdown")
                                            .route(web::post().to(api::kernel::queries::markdown)),
                                    )
                                    .service(
                                        web::resource("/generate_qr_code")
                                            .route(web::post().to(api::kernel::queries::generate_qr_code)),
                                    )
                                    .service(
                                        web::resource("/my_group_invitations")
                                            .route(web::post().to(api::kernel::queries::my_group_invitations)),
                                    )
                                    .service(
                                        web::resource("/my_sessions")
                                            .route(web::post().to(api::kernel::queries::my_sessions)),
                                    )
                                    .service(web::resource("/group").route(web::post().to(api::kernel::queries::group)))
                                    .service(web::resource("/group_with_members_and_invitations").route(
                                        web::post().to(api::kernel::queries::group_with_members_and_invitations),
                                    ))
                                    .service(
                                        web::resource("/stripe_public_key")
                                            .route(web::post().to(api::kernel::queries::stripe_public_key)),
                                    )
                                    .service(
                                        web::resource("/billing_information")
                                            .route(web::post().to(api::kernel::queries::billing_information)),
                                    )
                                    .service(
                                        web::resource("/checkout_session")
                                            .route(web::post().to(api::kernel::queries::checkout_session)),
                                    )
                                    .service(
                                        web::resource("/customer_portal")
                                            .route(web::post().to(api::kernel::queries::customer_portal)),
                                    ),
                            ),
                    )
                    // files
                    .service(
                        web::scope("/files")
                            .service(
                                web::scope("/commands")
                                    .service(
                                        web::resource("/move_files_to_trash")
                                            .route(web::post().to(api::files::commands::move_files_to_trash)),
                                    )
                                    .service(
                                        web::resource("/restore_files_from_trash")
                                            .route(web::post().to(api::files::commands::restore_files_from_trash)),
                                    )
                                    .service(
                                        web::resource("/empty_trash")
                                            .route(web::post().to(api::files::commands::empty_trash)),
                                    )
                                    .service(
                                        web::resource("/move_files")
                                            .route(web::post().to(api::files::commands::move_files)),
                                    )
                                    .service(
                                        web::resource("/create_folder")
                                            .route(web::post().to(api::files::commands::create_folder)),
                                    )
                                    .service(
                                        web::resource("/rename_file")
                                            .route(web::post().to(api::files::commands::rename_file)),
                                    )
                                    .service(
                                        web::resource("/complete_file_upload")
                                            .route(web::post().to(api::files::commands::complete_file_upload)),
                                    ),
                            )
                            .service(
                                web::scope("/queries")
                                    .service(web::resource("/file").route(web::post().to(api::files::queries::file)))
                                    .service(web::resource("/trash").route(web::post().to(api::files::queries::trash)))
                                    .service(
                                        web::resource("/file_download_url")
                                            .route(web::post().to(api::files::queries::file_download_url)),
                                    ),
                            ),
                    )
                    // analytics
                    // we use the /a shortcuts because otherwise the requests are blocked by adblockers
                    // even if they are legitimate
                    .service(
                        web::scope("/a")
                            .service(
                                web::scope("/events")
                                    .service(
                                        web::resource("/t")
                                            .route(web::post().to(api::analytics::commands::handle_track_event)),
                                    )
                                    .service(
                                        web::resource("/p")
                                            .route(web::post().to(api::analytics::commands::handle_page_event)),
                                    ),
                            )
                            .service(web::scope("/queries").service(
                                web::resource("/a").route(web::post().to(api::analytics::queries::analytics)),
                            )),
                    )
                    // inbox
                    .service(
                        web::scope("/inbox")
                            .service(
                                web::scope("/commands")
                                    .service(
                                        web::resource("/create_contact")
                                            .route(web::post().to(api::inbox::commands::create_contact)),
                                    )
                                    .service(
                                        web::resource("/delete_contact")
                                            .route(web::post().to(api::inbox::commands::delete_contact)),
                                    )
                                    .service(
                                        web::resource("/import_contacts")
                                            .route(web::post().to(api::inbox::commands::import_contacts)),
                                    )
                                    .service(
                                        web::resource("/update_contact")
                                            .route(web::post().to(api::inbox::commands::update_contact)),
                                    )
                                    .service(
                                        web::resource("/send_message")
                                            .route(web::post().to(api::inbox::commands::send_message)),
                                    )
                                    .service(
                                        web::resource("/send_chatbox_message")
                                            .route(web::post().to(api::inbox::commands::send_chatbox_message)),
                                    )
                                    .service(
                                        web::resource("/update_chatbox_preferences")
                                            .route(web::post().to(api::inbox::commands::update_chatbox_preferences)),
                                    )
                                    .service(
                                        web::resource("/link_chatbox_contact")
                                            .route(web::post().to(api::inbox::commands::link_chatbox_contact)),
                                    ),
                            )
                            .service(
                                web::scope("/queries")
                                    .service(
                                        web::resource("/contact").route(web::post().to(api::inbox::queries::contact)),
                                    )
                                    .service(
                                        web::resource("/contacts").route(web::post().to(api::inbox::queries::contacts)),
                                    )
                                    .service(
                                        web::resource("/chatbox_preferences")
                                            .route(web::post().to(api::inbox::queries::chatbox_preferences)),
                                    )
                                    .service(
                                        web::resource("/chatbox_messages")
                                            .route(web::post().to(api::inbox::queries::chatbox_messages)),
                                    )
                                    .service(web::resource("/inbox").route(web::post().to(api::inbox::queries::inbox)))
                                    .service(web::resource("/trash").route(web::post().to(api::inbox::queries::trash)))
                                    .service(
                                        web::resource("/archive").route(web::post().to(api::inbox::queries::archive)),
                                    )
                                    .service(web::resource("/spam").route(web::post().to(api::inbox::queries::spam))),
                            ),
                    )
                    // newsletter
                    .service(
                        web::scope("/newsletter")
                            .service(
                                web::scope("/commands")
                                    .service(
                                        web::resource("/create_list")
                                            .route(web::post().to(api::newsletter::commands::create_list)),
                                    )
                                    .service(
                                        web::resource("/create_message")
                                            .route(web::post().to(api::newsletter::commands::create_message)),
                                    )
                                    .service(
                                        web::resource("/delete_list")
                                            .route(web::post().to(api::newsletter::commands::delete_list)),
                                    )
                                    .service(
                                        web::resource("/delete_message")
                                            .route(web::post().to(api::newsletter::commands::delete_message)),
                                    )
                                    .service(
                                        web::resource("/send_message")
                                            .route(web::post().to(api::newsletter::commands::send_message)),
                                    )
                                    .service(
                                        web::resource("/send_test_message")
                                            .route(web::post().to(api::newsletter::commands::send_test_message)),
                                    )
                                    .service(
                                        web::resource("/update_list")
                                            .route(web::post().to(api::newsletter::commands::update_list)),
                                    )
                                    .service(
                                        web::resource("/update_message")
                                            .route(web::post().to(api::newsletter::commands::update_message)),
                                    )
                                    .service(
                                        web::resource("/subscribe_to_list")
                                            .route(web::post().to(api::newsletter::commands::subscribe_to_list)),
                                    )
                                    .service(
                                        web::resource("/unsubscribe_from_list")
                                            .route(web::post().to(api::newsletter::commands::unsubscribe_from_list)),
                                    ),
                            )
                            .service(
                                web::scope("/queries")
                                    .service(
                                        web::resource("/list").route(web::post().to(api::newsletter::queries::list)),
                                    )
                                    .service(
                                        web::resource("/lists").route(web::post().to(api::newsletter::queries::lists)),
                                    )
                                    .service(
                                        web::resource("/message")
                                            .route(web::post().to(api::newsletter::queries::message)),
                                    )
                                    .service(
                                        web::resource("/messages")
                                            .route(web::post().to(api::newsletter::queries::messages)),
                                    ),
                            ),
                    )
                    .default_service(
                        // 404 for GET request
                        web::resource("").to(api::p404),
                    ),
            )
            .service(
                // serve webapp
                actix_files::Files::new("/", &config.http.public_directory)
                    .index_file("index.html")
                    .prefer_utf8(true)
                    .default_handler(web::route().to(route_index)),
            )
    })
    .bind(endpoint)?
    .run()
    .await?;

    Ok(())
}
