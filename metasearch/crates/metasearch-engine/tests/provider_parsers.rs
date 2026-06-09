use metasearch_core::category::SearchCategory;
use metasearch_engine::{
    artifact_hub, datacite, dbpedia, doaj, europe_pmc, gitlab, hackernews, internet_archive,
    maven_central, met_museum, npm, nuget, nvd, open_food_facts, packagist, pubchem, rubygems,
    semantic_scholar, sourcehut, stackexchange, zenodo,
};

#[test]
fn rubygems_parser_shapes_package_results() {
    let body = r#"
    [
      {
        "name": "rails",
        "info": "Full-stack web framework",
        "version": "8.0.0",
        "project_uri": "https://rubygems.org/gems/rails",
        "downloads": 123456
      }
    ]
    "#;

    let results = rubygems::parse_rubygems_results(body).expect("valid RubyGems JSON");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "rails");
    assert_eq!(results[0].url, "https://rubygems.org/gems/rails");
    assert_eq!(results[0].engine, "rubygems");
    assert_eq!(results[0].engine_rank, 1);
    assert_eq!(results[0].category, SearchCategory::IT.to_string());
    assert!(results[0].content.contains("8.0.0"));
}

#[test]
fn packagist_parser_shapes_package_results() {
    let body = r#"
    {
      "results": [
        {
          "name": "monolog/monolog",
          "description": "Sends your logs to files, sockets, inboxes and more",
          "url": "https://packagist.org/packages/monolog/monolog",
          "downloads": 123456,
          "favers": 789
        }
      ],
      "total": 1
    }
    "#;

    let results = packagist::parse_packagist_results(body).expect("valid Packagist JSON");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "monolog/monolog");
    assert_eq!(
        results[0].url,
        "https://packagist.org/packages/monolog/monolog"
    );
    assert_eq!(results[0].engine, "packagist");
    assert_eq!(results[0].engine_rank, 1);
    assert_eq!(results[0].category, SearchCategory::IT.to_string());
    assert!(results[0].content.contains("downloads"));
}

#[test]
fn internet_archive_parser_shapes_archive_items() {
    let body = r#"
    {
      "response": {
        "docs": [
          {
            "identifier": "rustbook",
            "title": "The Rust Programming Language",
            "description": "An archived Rust book",
            "mediatype": "texts",
            "creator": ["Rust Project"],
            "year": "2021"
          }
        ]
      }
    }
    "#;

    let results = internet_archive::parse_internet_archive_results(body)
        .expect("valid Internet Archive JSON");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "The Rust Programming Language");
    assert_eq!(results[0].url, "https://archive.org/details/rustbook");
    assert_eq!(results[0].engine, "internet_archive");
    assert_eq!(results[0].engine_rank, 1);
    assert_eq!(results[0].category, SearchCategory::General.to_string());
    assert_eq!(
        results[0].thumbnail.as_deref(),
        Some("https://archive.org/services/img/rustbook")
    );
}

#[test]
fn doaj_parser_shapes_article_results() {
    let body = r#"
    {
      "results": [
        {
          "id": "abc123",
          "bibjson": {
            "title": "Open access metasearch",
            "abstract": "A paper about open indexes",
            "year": "2025",
            "journal": { "title": "Journal of Open Search" },
            "link": [
              { "url": "https://example.org/article", "type": "fulltext" }
            ],
            "identifier": [
              { "type": "doi", "id": "10.1234/example" }
            ],
            "author": [
              { "name": "Ada Lovelace" }
            ]
          }
        }
      ]
    }
    "#;

    let results = doaj::parse_doaj_results(body).expect("valid DOAJ JSON");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Open access metasearch");
    assert_eq!(results[0].url, "https://example.org/article");
    assert_eq!(results[0].engine, "doaj");
    assert_eq!(results[0].engine_rank, 1);
    assert_eq!(results[0].category, SearchCategory::Science.to_string());
    assert!(results[0].content.contains("Ada Lovelace"));
}

#[test]
fn maven_central_parser_shapes_package_results() {
    let body = r#"
    {
      "response": {
        "docs": [
          {
            "id": "org.example:demo",
            "g": "org.example",
            "a": "demo",
            "latestVersion": "1.2.3",
            "p": "jar",
            "versionCount": 12
          }
        ]
      }
    }
    "#;

    let results =
        maven_central::parse_maven_central_results(body).expect("valid Maven Central JSON");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "org.example:demo");
    assert_eq!(
        results[0].url,
        "https://central.sonatype.com/artifact/org.example/demo"
    );
    assert_eq!(results[0].engine, "maven_central");
    assert_eq!(results[0].engine_rank, 1);
    assert_eq!(results[0].category, SearchCategory::IT.to_string());
    assert!(results[0].content.contains("1.2.3"));
}

#[test]
fn nuget_parser_shapes_package_results() {
    let body = r#"
    {
      "data": [
        {
          "id": "Newtonsoft.Json",
          "version": "13.0.3",
          "description": "Json.NET is a popular high-performance JSON framework",
          "authors": ["James Newton-King"],
          "projectUrl": "https://www.newtonsoft.com/json",
          "totalDownloads": 123456
        }
      ]
    }
    "#;

    let results = nuget::parse_nuget_results(body).expect("valid NuGet JSON");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Newtonsoft.Json");
    assert_eq!(
        results[0].url,
        "https://www.nuget.org/packages/Newtonsoft.Json"
    );
    assert_eq!(results[0].engine, "nuget");
    assert_eq!(results[0].engine_rank, 1);
    assert_eq!(results[0].category, SearchCategory::IT.to_string());
    assert!(results[0].content.contains("13.0.3"));
}

#[test]
fn npm_official_registry_parser_shapes_package_results() {
    let body = r#"
    {
      "objects": [
        {
          "package": {
            "name": "vite",
            "version": "7.0.0",
            "description": "Native-ESM powered web dev build tool",
            "links": {
              "npm": "https://www.npmjs.com/package/vite",
              "repository": "https://github.com/vitejs/vite"
            },
            "publisher": { "username": "vitebot" }
          },
          "score": { "final": 0.95 }
        }
      ],
      "total": 1
    }
    "#;

    let results = npm::parse_npm_registry_results(body).expect("valid npm registry JSON");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "vite");
    assert_eq!(results[0].url, "https://www.npmjs.com/package/vite");
    assert_eq!(results[0].engine, "npm");
    assert_eq!(results[0].engine_rank, 1);
    assert_eq!(results[0].category, SearchCategory::IT.to_string());
    assert!(results[0].content.contains("7.0.0"));
    assert!(results[0].content.contains("vitebot"));
}

#[test]
fn europe_pmc_parser_shapes_article_results() {
    let body = r#"
    {
      "resultList": {
        "result": [
          {
            "id": "12345",
            "source": "MED",
            "title": "Open biomedical search",
            "authorString": "Ada Lovelace",
            "journalTitle": "Journal of Open Biology",
            "pubYear": "2025",
            "doi": "10.1234/pmc",
            "abstractText": "A biomedical search paper"
          }
        ]
      }
    }
    "#;

    let results = europe_pmc::parse_europe_pmc_results(body).expect("valid Europe PMC JSON");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Open biomedical search");
    assert_eq!(results[0].url, "https://doi.org/10.1234/pmc");
    assert_eq!(results[0].engine, "europe_pmc");
    assert_eq!(results[0].engine_rank, 1);
    assert_eq!(results[0].category, SearchCategory::Science.to_string());
    assert!(results[0].content.contains("Ada Lovelace"));
}

#[test]
fn dbpedia_parser_shapes_entity_results() {
    let body = r#"
    {
      "docs": [
        {
          "label": ["Rust"],
          "resource": ["http://dbpedia.org/resource/Rust_(programming_language)"],
          "comment": ["A systems programming language"]
        }
      ]
    }
    "#;

    let results = dbpedia::parse_dbpedia_results(body).expect("valid DBpedia JSON");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Rust");
    assert_eq!(
        results[0].url,
        "http://dbpedia.org/resource/Rust_(programming_language)"
    );
    assert_eq!(results[0].engine, "dbpedia");
    assert_eq!(results[0].engine_rank, 1);
    assert_eq!(results[0].category, SearchCategory::General.to_string());
}

#[test]
fn open_food_facts_parser_shapes_product_results() {
    let body = r#"
    {
      "products": [
        {
          "product_name": "Example Cereal",
          "code": "1234567890",
          "brands": "Example Brand",
          "categories": "Breakfast cereals",
          "url": "https://world.openfoodfacts.org/product/1234567890/example-cereal",
          "image_front_thumb_url": "https://images.openfoodfacts.org/example.jpg"
        }
      ]
    }
    "#;

    let results =
        open_food_facts::parse_open_food_facts_results(body).expect("valid Open Food Facts JSON");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Example Cereal");
    assert_eq!(results[0].engine, "open_food_facts");
    assert_eq!(results[0].engine_rank, 1);
    assert_eq!(results[0].category, SearchCategory::General.to_string());
    assert_eq!(
        results[0].thumbnail.as_deref(),
        Some("https://images.openfoodfacts.org/example.jpg")
    );
}

#[test]
fn met_museum_parser_shapes_object_results() {
    let objects = vec![
        r#"
      {
        "objectID": 42,
        "title": "Vase",
        "objectURL": "https://www.metmuseum.org/art/collection/search/42",
        "artistDisplayName": "Unknown artist",
        "objectDate": "1880",
        "medium": "Ceramic",
        "primaryImageSmall": "https://images.metmuseum.org/CRDImages/example.jpg"
      }
    "#,
    ];

    let results =
        met_museum::parse_met_object_results(&objects).expect("valid The Met object JSON");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Vase");
    assert_eq!(
        results[0].url,
        "https://www.metmuseum.org/art/collection/search/42"
    );
    assert_eq!(results[0].engine, "met_museum");
    assert_eq!(results[0].engine_rank, 1);
    assert_eq!(results[0].category, SearchCategory::Images.to_string());
}

#[test]
fn datacite_parser_shapes_doi_results() {
    let body = r#"
    {
      "data": [
        {
          "id": "10.5281/zenodo.123",
          "attributes": {
            "doi": "10.5281/zenodo.123",
            "titles": [{ "title": "Open dataset search" }],
            "creators": [{ "name": "Ada Lovelace" }],
            "publisher": "Zenodo",
            "publicationYear": 2025,
            "url": "https://zenodo.org/records/123",
            "descriptions": [
              { "description": "A searchable open dataset", "descriptionType": "Abstract" }
            ],
            "types": { "resourceTypeGeneral": "Dataset" }
          }
        }
      ]
    }
    "#;

    let results = datacite::parse_datacite_results(body).expect("valid DataCite JSON");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Open dataset search");
    assert_eq!(results[0].url, "https://zenodo.org/records/123");
    assert_eq!(results[0].engine, "datacite");
    assert_eq!(results[0].engine_rank, 1);
    assert_eq!(results[0].category, SearchCategory::Science.to_string());
    assert!(results[0].content.contains("Ada Lovelace"));
    assert!(results[0].content.contains("Dataset"));
}

#[test]
fn zenodo_parser_shapes_record_results() {
    let body = r#"
    {
      "hits": {
        "hits": [
          {
            "id": 6414629,
            "doi_url": "https://doi.org/10.5281/zenodo.6414629",
            "links": { "self_html": "https://zenodo.org/records/6414629" },
            "metadata": {
              "title": "Rust Web Server",
              "publication_date": "2022-04-01",
              "description": "<p>A small Rust web server.</p>",
              "resource_type": { "title": "Software" },
              "creators": [{ "name": "Bohdan Tsap" }],
              "keywords": ["rust", "web server"],
              "version": "0.0.7"
            },
            "stats": { "views": 196, "downloads": 25 }
          }
        ]
      }
    }
    "#;

    let results = zenodo::parse_zenodo_results(body).expect("valid Zenodo JSON");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Rust Web Server");
    assert_eq!(results[0].url, "https://zenodo.org/records/6414629");
    assert_eq!(results[0].engine, "zenodo");
    assert_eq!(results[0].engine_rank, 1);
    assert_eq!(results[0].category, SearchCategory::Science.to_string());
    assert!(results[0].content.contains("Bohdan Tsap"));
    assert!(results[0].content.contains("Software"));
}

#[test]
fn artifact_hub_parser_shapes_package_results() {
    let body = r#"
    {
      "packages": [
        {
          "package_id": "bec45f52-fe90-46a1-b6eb-a3dcd9bb203f",
          "name": "nginx",
          "description": "NGINX Open Source chart",
          "version": "24.0.2",
          "app_version": "1.31.1",
          "stars": 104,
          "repository": {
            "name": "bitnami",
            "display_name": "Bitnami",
            "verified_publisher": true
          }
        }
      ]
    }
    "#;

    let results = artifact_hub::parse_artifact_hub_results(body).expect("valid Artifact Hub JSON");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "nginx");
    assert_eq!(
        results[0].url,
        "https://artifacthub.io/packages/search?ts_query_web=nginx"
    );
    assert_eq!(results[0].engine, "artifact_hub");
    assert_eq!(results[0].engine_rank, 1);
    assert_eq!(results[0].category, SearchCategory::IT.to_string());
    assert!(results[0].content.contains("Bitnami"));
    assert!(results[0].content.contains("stars: 104"));
}

#[test]
fn nvd_parser_shapes_official_cve_results() {
    let body = r#"
    {
      "vulnerabilities": [
        {
          "cve": {
            "id": "CVE-2017-1000430",
            "published": "2018-01-02T20:29:00.313",
            "vulnStatus": "Modified",
            "descriptions": [
              {
                "lang": "en",
                "value": "rust-base64 version <= 0.5.1 is vulnerable to a buffer overflow"
              }
            ],
            "metrics": {
              "cvssMetricV31": [
                {
                  "cvssData": {
                    "baseScore": 9.8,
                    "baseSeverity": "CRITICAL"
                  }
                }
              ]
            },
            "weaknesses": [
              {
                "description": [
                  { "lang": "en", "value": "CWE-119" }
                ]
              }
            ]
          }
        }
      ]
    }
    "#;

    let results = nvd::parse_nvd_results(body).expect("valid NVD CVE JSON");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "CVE-2017-1000430");
    assert_eq!(
        results[0].url,
        "https://nvd.nist.gov/vuln/detail/CVE-2017-1000430"
    );
    assert_eq!(results[0].engine, "nvd");
    assert_eq!(results[0].engine_rank, 1);
    assert_eq!(results[0].category, SearchCategory::IT.to_string());
    assert!(results[0].content.contains("CRITICAL"));
    assert!(results[0].content.contains("CWE-119"));
    assert!(results[0].published_date.is_some());
}

#[test]
fn pubchem_parser_shapes_compound_results() {
    let body = r#"
    {
      "PropertyTable": {
        "Properties": [
          {
            "CID": 2244,
            "MolecularFormula": "C9H8O4",
            "MolecularWeight": "180.16",
            "ConnectivitySMILES": "CC(=O)OC1=CC=CC=C1C(=O)O",
            "InChIKey": "BSYNRYMUTXBXSQ-UHFFFAOYSA-N",
            "IUPACName": "2-acetyloxybenzoic acid"
          }
        ]
      }
    }
    "#;

    let results = pubchem::parse_pubchem_results(body).expect("valid PubChem JSON");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "2-acetyloxybenzoic acid");
    assert_eq!(
        results[0].url,
        "https://pubchem.ncbi.nlm.nih.gov/compound/2244"
    );
    assert_eq!(results[0].engine, "pubchem");
    assert_eq!(results[0].engine_rank, 1);
    assert_eq!(results[0].category, SearchCategory::Science.to_string());
    assert!(results[0].content.contains("C9H8O4"));
    assert!(results[0].content.contains("180.16"));
}

#[test]
fn hackernews_parser_shapes_story_results() {
    let body = r#"
    {
      "hits": [
        {
          "objectID": "123",
          "title": "Rust at Scale",
          "url": "https://example.com/rust",
          "points": 42,
          "num_comments": 7,
          "author": "ada"
        }
      ]
    }
    "#;

    let results = hackernews::parse_hackernews_results(body).expect("valid HN JSON");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Rust at Scale");
    assert_eq!(results[0].url, "https://news.ycombinator.com/item?id=123");
    assert_eq!(results[0].engine, "hackernews");
    assert_eq!(results[0].engine_rank, 1);
    assert_eq!(results[0].category, SearchCategory::IT.to_string());
    assert!(results[0].content.contains("points: 42"));
}

#[test]
fn gitlab_parser_shapes_project_results() {
    let body = r#"
    [
      {
        "name": "metasearch",
        "web_url": "https://gitlab.com/example/metasearch",
        "description": "Federated search",
        "star_count": 12,
        "forks_count": 3,
        "namespace": { "name": "example" },
        "avatar_url": "https://gitlab.com/example/avatar.png"
      }
    ]
    "#;

    let results = gitlab::parse_gitlab_results(body).expect("valid GitLab JSON");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "metasearch");
    assert_eq!(results[0].url, "https://gitlab.com/example/metasearch");
    assert_eq!(results[0].engine, "gitlab");
    assert_eq!(results[0].engine_rank, 1);
    assert_eq!(results[0].category, SearchCategory::IT.to_string());
    assert!(results[0].content.contains("stars: 12"));
}

#[test]
fn stackexchange_parser_shapes_question_results() {
    let body = r#"
    {
      "items": [
        {
          "question_id": 99,
          "title": "How to parse JSON in Rust?",
          "tags": ["rust", "serde"],
          "owner": { "display_name": "Ferris" },
          "is_answered": true,
          "score": 5,
          "answer_count": 2,
          "view_count": 100
        }
      ]
    }
    "#;

    let results = stackexchange::parse_stackexchange_results(body, "stackoverflow")
        .expect("valid Stack Exchange JSON");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "How to parse JSON in Rust?");
    assert_eq!(results[0].url, "https://stackoverflow.com/q/99");
    assert_eq!(results[0].engine, "stackexchange");
    assert_eq!(results[0].engine_rank, 1);
    assert_eq!(results[0].category, SearchCategory::IT.to_string());
    assert!(results[0].content.contains("2 answers"));
}

#[test]
fn semantic_scholar_parser_shapes_official_graph_results() {
    let body = r#"
    {
      "total": 1,
      "data": [
        {
          "paperId": "abc123",
          "title": "Open scholarly search",
          "url": "https://www.semanticscholar.org/paper/abc123",
          "abstract": "A paper about search",
          "year": 2025,
          "venue": "Journal of Search",
          "citationCount": 9,
          "authors": [
            { "name": "Ada Lovelace" },
            { "name": "Grace Hopper" }
          ],
          "externalIds": { "DOI": "10.1234/search" },
          "openAccessPdf": { "url": "https://example.org/paper.pdf" }
        }
      ]
    }
    "#;

    let results =
        semantic_scholar::parse_semantic_scholar_results(body).expect("valid S2 Graph JSON");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Open scholarly search");
    assert_eq!(
        results[0].url,
        "https://www.semanticscholar.org/paper/abc123"
    );
    assert_eq!(results[0].engine, "semantic_scholar");
    assert_eq!(results[0].engine_rank, 1);
    assert_eq!(results[0].category, SearchCategory::Science.to_string());
    assert!(results[0].content.contains("Ada Lovelace"));
    assert!(results[0].content.contains("10.1234/search"));
}

#[test]
fn sourcehut_parser_shapes_fixture_results() {
    let body = r#"
    <html>
      <body>
        <div class="event-list">
          <div class="event">
            <h4>
              <a href="/~owner/">owner</a>
              <a href="/~owner/project">project</a>
            </h4>
            <p>Small project description</p>
          </div>
        </div>
      </body>
    </html>
    "#;

    let results = sourcehut::parse_sourcehut_results(body).expect("valid SourceHut HTML");

    assert_eq!(results.len(), 1);
    assert!(results[0].title.contains("project"));
    assert_eq!(results[0].url, "https://sr.ht/~owner/project");
    assert_eq!(results[0].engine, "sourcehut");
    assert_eq!(results[0].engine_rank, 1);
    assert_eq!(results[0].category, SearchCategory::IT.to_string());
}
