# Provider Access Models

DX metasearch distinguishes adapter coverage from live provider health. A registered adapter means code exists; it does not prove that the upstream provider is configured, reachable, or currently returning results.

## Access Model Classes

| Class | Meaning |
| --- | --- |
| `no_key_open_endpoint` | Documented public endpoint with no local credential requirement. Still not unlimited unless upstream docs explicitly say so. |
| `no_key_rate_limited` | Documented public endpoint with no local credential requirement, but callers should expect quota, backoff, or user-agent rules. |
| `optional_api_key` | Public unauthenticated access exists, but a key improves quota or behavior. |
| `requires_api_key` | Adapter must be skipped until a key or token is configured. |
| `self_hosted_instance` | Adapter must be skipped until a trusted instance/base URL is configured. |
| `html_scraper_brittle` | Adapter parses upstream HTML or page scripts and should not be treated as production live health without a recent probe. |

## Newly Added Adapters

| Adapter | Access model | Documentation reference |
| --- | --- | --- |
| `internet_archive` | `no_key_rate_limited` | Official advanced search JSON endpoint: https://archive.org/advancedsearch.php |
| `rubygems` | `no_key_rate_limited` | Official RubyGems.org API guide: https://guides.rubygems.org/rubygems-org-api/ |
| `packagist` | `no_key_open_endpoint` | Official Packagist API docs: https://packagist.org/apidoc |
| `doaj` | `no_key_rate_limited` | Official DOAJ metadata help says metadata is available through its API and public data services: https://doaj.org/docs/faq/ |
| `maven_central` | `no_key_rate_limited` | Official Sonatype Central REST API docs and 429 policy: https://central.sonatype.org/search/rest-api-guide/ |
| `nuget` | `no_key_open_endpoint` | Official NuGet V3 search service docs: https://learn.microsoft.com/en-us/nuget/api/search-query-service-resource |
| `europe_pmc` | `no_key_open_endpoint` | Official Europe PMC RESTful Web Service docs: https://dev.europepmc.org/RestfulWebService |
| `dbpedia` | `no_key_open_endpoint` | Official DBpedia Lookup API docs: https://www.dbpedia.org/resources/lookup/ |
| `open_food_facts` | `no_key_rate_limited`, disabled by default pending live reliability proof | Official Open Food Facts API docs: https://openfoodfacts.github.io/openfoodfacts-server/api/ |
| `met_museum` | `no_key_rate_limited` | Official Met Collection API docs: https://metmuseum.github.io/ |
| `datacite` | `no_key_rate_limited` | Official DataCite public REST API docs: https://support.datacite.org/docs/api |
| `zenodo` | `no_key_rate_limited` | Official Zenodo REST API and rate-limit docs: https://developers.zenodo.org/ |
| `artifact_hub` | `no_key_rate_limited` | Official Artifact Hub OpenAPI docs: https://artifacthub.github.io/hub/api/ |
| `nvd` | `optional_api_key` | Official NVD CVE API 2.0 docs and API-key/rate-limit guidance: https://nvd.nist.gov/developers/vulnerabilities |
| `pubchem` | `no_key_rate_limited` | Official PubChem PUG REST docs and usage policy: https://pubchem.ncbi.nlm.nih.gov/docs/pug-rest |

## Hardened Existing Adapters

| Adapter | Access model | Documentation reference |
| --- | --- | --- |
| `semantic_scholar` | `optional_api_key` | Official Semantic Scholar Academic Graph API docs: https://api.semanticscholar.org/api-docs/ |
| `hackernews` | `no_key_rate_limited` | HN Algolia API docs: https://hn.algolia.com/api |
| `gitlab` | `no_key_rate_limited` | Official GitLab Projects API docs: https://docs.gitlab.com/api/projects/#list-all-projects |
| `stackexchange` | `no_key_rate_limited` | Official Stack Exchange API docs: https://api.stackexchange.com/docs |
| `npm` | `no_key_rate_limited` | Official npm registry search API docs: https://github.com/npm/registry/blob/main/docs/REGISTRY-API.md |
| `Google News` / `google_news` | `no_key_rate_limited` RSS feed | Public Google News RSS search endpoint: https://news.google.com/rss/search |
| `sourcehut` | `html_scraper_brittle`, disabled by default | SourceHut docs are available at https://docs.sourcehut.org/, but this adapter still parses project-search HTML. |
| `pypi` | `html_scraper_brittle`, disabled by default | PyPI no longer provides a supported broad package-search API; this adapter only scrapes search-result HTML. XML-RPC search deprecation reference: https://warehouse.pypa.io/api-reference/xml-rpc.html |
| `searchcode_code` | `not_acceptable`, disabled by default | Current Searchcode OpenAPI is repo-scoped rather than the legacy global code-search endpoint: https://searchcode.com/openapi.json |

## Operator Surfaces

- `/api/v1/engines` reports registered adapters, default-enabled state, configured state, effective enabled state, access model, implementation strategy, config requirements, skip reason, and health/probe state.
- `/api/v1/status`, `/health`, and `/readyz` include provider summaries. Cold-start provider status means no recent probe has succeeded yet; `partially_healthy` means some adapters have recent success but not all effective adapters are proven healthy. Readiness means the service has usable search capacity, not that every adapter has been live-probed.
- `metasearch probe --allow-network --engines <ids> --query <query>` is the lightweight live check. Use selected engines, low timeouts, and low concurrency; do not treat all-adapter count as all-provider health.
