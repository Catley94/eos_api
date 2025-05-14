mod api; // Declare a module named `api` (file: api.rs or folder: api/)
mod unreal; // Declare a module named `unreal`

use egs_api::EpicGames; // Import the EpicGames struct from the egs_api crate/module
use std::io::{self}; // Import standard input/output utilities
use std::collections::HashSet; // Import HashSet for storing unique values
use std::collections::HashMap; // Import HashMap for key-value storage
use std::thread::sleep; // Import sleep function for pausing execution
use std::time::Duration; // Import Duration for specifying sleep time
use egs_api::api::types::epic_asset::EpicAsset; // Import EpicAsset struct
use egs_api::api::error::EpicAPIError; // Import EpicAPIError enum

#[tokio::main] // Macro to mark the async main function for the Tokio runtime
async fn main() {
    env_logger::init(); // Initialize the logger for debugging output

    // Try to open the Epic Games login page in the user's default web browser
    if webbrowser::open("https://www.epicgames.com/id/login?redirectUrl=https%3A%2F%2Fwww.epicgames.com%2Fid%2Fapi%2Fredirect%3FclientId%3D34a02cf8f4414e29b15921876da36f9a%26responseType%3Dcode").is_err() {
        // If opening the browser fails, print the login URL for manual navigation
        println!("Please go to https://www.epicgames.com/id/login?redirectUrl=https%3A%2F%2Fwww.epicgames.com%2Fid%2Fapi%2Fredirect%3FclientId%3D34a02cf8f4414e29b15921876da36f9a%26responseType%3Dcode")
    }
    println!("Please enter the 'authorizationCode' value from the JSON response");
    let mut sid = String::new(); // Create a mutable String to store the authorization code
    let stdin = io::stdin(); // Get a handle to standard input
    stdin.read_line(&mut sid).unwrap(); // Read a line from the user into `sid`
    sid = sid.trim().to_string(); // Remove whitespace from the input and convert to String
    sid = sid.replace(|c: char| c == '"', ""); // Remove any double quotes from the input

    let mut egs = EpicGames::new(); // Create a new EpicGames API client
    println!("Using Auth Code: {}", sid); // Print the authorization code being used

    // Authenticate with the EpicGames API using the provided authorization code
    if egs.auth_code(None, Some(sid)).await {
        println!("Logged in"); // Print success message if authentication succeeds
    }

    egs.login().await; // Log in to the EpicGames API
    let details = egs.account_details().await; // Fetch account details from the API
    println!("Account details: {:?}", details); // Print the account details

    // Fetch additional account info using the account ID
    let info = egs
        .account_ids_details(vec![egs.user_details().account_id.unwrap_or_default()])
        .await;
    println!("Account info: {:?}", info); // Print the account info

    // The following commented code would fetch and print friends, if uncommented
    // let friends = egs.account_friends(true).await;
    // println!("Friends: {:?}", friends);

    // Handle the result of fetching account details
    match details {
        None => {} // If no details, do nothing
        Some(info) => {
            let assets = egs.fab_library_items(info.id).await; // Fetch the user's asset library
            match assets {
                None => {
                    println!("No assets found"); // Print if no assets are found
                }
                Some(ass) => {
                    println!("Library items: {:?}", ass.results.len()); // Print the number of library items
                    // Iterate over each asset in the results
                    for asset in ass.results.iter() {
                        // For each version of the asset's project
                        for version in asset.project_versions.iter() {
                            loop {
                                // Attempt to fetch the manifest for this asset version
                                let manifest = egs.fab_asset_manifest(
                                    &version.artifact_id,
                                    &asset.asset_namespace,
                                    &asset.asset_id,
                                    None,
                                ).await;
                                match manifest {
                                    Ok(manifest) => {
                                        // If successful, print a message and break out of the loop
                                        println!("OK Manifest for {} - {}", asset.title, version.artifact_id);
                                        break;
                                    }
                                    Err(e) => {
                                        match e {
                                            EpicAPIError::FabTimeout => {
                                                // If a timeout error occurs, retry the loop after a delay
                                                // sleep(Duration::from_millis(1000)).await; // (commented out)
                                                continue;
                                            }
                                            _ => {}
                                        }
                                        // For other errors, print a message and break out of the loop
                                        println!("NO Manifest for {} - {}", asset.title, version.artifact_id);
                                        break;
                                    }
                                }
                            }
                            // Sleep for 1 second between requests to avoid rate limiting
                            sleep(Duration::from_millis(1000));
                        }
                    }
                }
            }
        }
    }

    // Example: Fetch the manifest for a specific asset by its IDs
    let manifest = egs
        .fab_asset_manifest(
            "KiteDemo473",
            "89efe5924d3d467c839449ab6ab52e7f",
            "28166226c38a4ff3aa28bbe87dcbbe5b",
            None,
        )
        .await;
    println!("Kite Demo Manifest: {:#?}", manifest); // Print the manifest details
    
    
    // TODO Get asset from account
    // Create manifest as above from asset from account
    
    // Look at what EpicDownloadManager is in Epic-Asset-Manager
    // self.imp(); // Self being EpicDownloadManager
    
    

    // If manifest fetch was successful, iterate over its distribution URLs
    if let Ok(manif) = manifest {
        for man in manif.iter() {
            for url in man.distribution_point_base_urls.iter() {
                println!("Trying to get download manifest from {}", url); // Print which URL is being tried
                let dm = egs.fab_download_manifest(man.clone(), url).await; // Attempt to download the manifest
                match dm {
                    Ok(d) => {
                        println!("Got download manifest from {}", url); // Print success message
                        println!("Expected Hash: {}", man.manifest_hash); // Print expected hash
                        println!("Download Hash: {}", d.custom_field("DownloadedManifestHash").unwrap_or_default()); // Print hash from download
                    }
                    Err(_) => {} // Do nothing if download fails
                }
            }
        }
    };

    // The following commented code would open a browser for game token exchange, if uncommented
    // let code = egs.game_token().await;
    // if let Some(c) = code {
    //     let authorized_url = format!("https://www.epicgames.com/id/exchange?exchangeCode={}&redirectUrl=https%3A%2F%2Fwww.unrealengine.com%2Fdashboard%3Flang%3Den", c.code);
    //     if webbrowser::open(&authorized_url).is_err() {
    //         println!("Please go to {}", authorized_url)
    //     }
    // }

    // Fetch all assets from the account
    let assets = egs.list_assets(None, None).await;

    // Create two HashMaps to categorize the assets: Unreal Engine and non-Unreal Engine
    let mut ueasset_map: HashMap<String, HashMap<String, EpicAsset>> = HashMap::new();
    let mut non_ueasset_map: HashMap<String, HashMap<String, EpicAsset>> = HashMap::new();

    // Categorize each asset into the appropriate HashMap
    for asset in assets {
        if asset.namespace == "ue" {
            // If the asset is an Unreal Engine asset
            // println!("----------------------------------------");
            // println!("Unreal Engine Asset: {:?} ", asset);
            // println!("----------------------------------------");
            if !ueasset_map.contains_key(&asset.catalog_item_id.clone()) {
                // If this catalog item ID is not yet in the map, insert a new entry
                ueasset_map.insert(asset.catalog_item_id.clone(), HashMap::new());
            };
            // Insert the asset into the inner HashMap keyed by app name
            match ueasset_map.get_mut(&asset.catalog_item_id.clone()) {
                None => {}
                Some(old) => {
                    old.insert(asset.app_name.clone(), asset.clone());
                }
            };
        } else {
            // If the asset is NOT an Unreal Engine asset
            // println!("+++++++++++++++++++++++++++++++++++++++++");
            // println!("NON-Unreal Engine Asset: {:?} ", asset);
            // println!("+++++++++++++++++++++++++++++++++++++++++");
            if !non_ueasset_map.contains_key(&asset.catalog_item_id.clone()) {
                // If this catalog item ID is not yet in the map, insert a new entry
                non_ueasset_map.insert(asset.catalog_item_id.clone(), HashMap::new());
            };
            // Insert the asset into the inner HashMap keyed by app name
            match non_ueasset_map.get_mut(&asset.catalog_item_id.clone()) {
                None => {}
                Some(old) => {
                    old.insert(asset.app_name.clone(), asset.clone());
                }
            };
        }
    }

    // Print summary statistics about the assets
    println!("Got {} assets", ueasset_map.len() + non_ueasset_map.len());
    println!("From that {} unreal assets", ueasset_map.len());
    println!("From that {} non unreal assets", non_ueasset_map.len());

    println!("Getting the last asset in ueasset_map HashMap's metadata");

    // Get the last Unreal Engine asset in the map for testing
    let test_asset = ueasset_map
        .values()
        .last()
        .unwrap()
        .values()
        .last()
        .unwrap()
        .to_owned();
    get_asset(&mut egs, &test_asset).await; // Fetch and process the test asset
    println!("{:#?}", test_asset.clone()); // Print the test asset details

    println!("Getting the asset info");

    // Create a HashSet to store all unique categories from assets
    let mut categories: HashSet<String> = HashSet::new();

    // Iterate over all non-Unreal Engine assets to collect their categories
    for (_guid, asset) in non_ueasset_map.clone() {
        match egs
            .asset_info(asset.values().last().unwrap().to_owned())
            .await
        {
            None => {}
            Some(info) => {
                // For each category in the asset info, insert its path into the categories set
                for category in info.categories.unwrap() {
                    categories.insert(category.path);
                }
            }
        };
    }
    // Convert the HashSet of categories to a sorted Vec
    let mut cat: Vec<String> = categories.into_iter().collect();
    cat.sort();
    // Print each category
    for category in cat {
        println!("Category: {}", category);
    }
    // Fetch asset info for the test asset
    let _asset_info = egs.asset_info(test_asset.clone()).await;
    println!("Getting ownership token");
    egs.ownership_token(test_asset.clone()).await; // Get ownership token for the test asset
    println!("Getting the game token");
    egs.game_token().await; // Get the game token
    println!("Getting the entitlements");
    egs.user_entitlements().await; // Get user entitlements
    println!("Getting the library items");
    egs.library_items(true).await; // Get library items

    println!("Getting Asset manifest");
    // Fetch the asset manifest for the test asset
    let manifest = egs
        .asset_manifest(
            None,
            None,
            Some(test_asset.namespace.clone()),
            Some(test_asset.catalog_item_id.clone()),
            Some(test_asset.app_name.clone()),
        )
        .await;
    println!("{:?}", manifest); // Print the manifest

    
    
    // Fetch the download manifest for the asset
    let download_manifest = egs.asset_download_manifests(manifest.unwrap()).await;
    println!("{:?}", download_manifest);
}

// Async function to get the manifest for a given asset
async fn get_asset(egs: &mut EpicGames, test_asset: &EpicAsset) {
    egs.asset_manifest(
        None,
        None,
        Some(test_asset.namespace.clone()),
        Some(test_asset.catalog_item_id.clone()),
        Some(test_asset.app_name.clone()),
    )
        .await;
}
