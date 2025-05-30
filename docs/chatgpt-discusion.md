Rust-Based MCP Server Generation from OpenAPI: Agent-Oriented Design and Authentication

Introduction

The Model Context Protocol (MCP) allows AI agents to use tools and services via standardized interfaces. To integrate existing REST APIs into multi-agent workflows (e.g. Socialings AI), we can create Rust-based MCP servers that wrap those APIs based on their OpenAPI (Swagger) specifications. However, simply exposing every endpoint “as-is” is often not ideal for an AI agent. This report discusses how to translate developer-focused OpenAPI specs into agent-optimized endpoints and instructions, how to detect and handle authentication automatically (API keys, OAuth2, etc.), and the viability/challenges of wrapping major social media APIs (Facebook, LinkedIn, Twitter/X) in this manner. We also outline architectural approaches, code generation strategies, and relevant Rust tools/crates for building these MCP wrappers.

From OpenAPI to Agent-Optimized Endpoints

OpenAPI documents describe low-level REST operations (CRUD on resources, etc.) intended for human developers. An AI agent, by contrast, benefits from a higher-level, task-oriented interface with clear guidance. Key considerations in translating OpenAPI endpoints into agent-facing MCP tools include:
	•	Selecting Relevant Endpoints: Large APIs can have hundreds of operations, which would overwhelm an LLM agent ￼. It’s crucial to prune aggressively – expose only the endpoints that represent distinct, useful capabilities for the agent ￼ ￼. For example, a GitHub API has ~600 endpoints; an agent may only need a handful (list repos, create issue, etc.). Filtering by tag or path (as some generators allow) helps limit the toolset to essentials. In practice, developers often whitelist specific paths or tags when generating the MCP server ￼.
	•	Merging or Abstracting Operations: REST APIs are resource-centric, whereas agents think in terms of tasks. An agent might struggle if it has to combine multiple fine-grained calls on its own ￼. Where feasible, design higher-level MCP tools that encapsulate multi-step workflows behind one agent command ￼ ￼. For instance, instead of separate “start transaction”, “execute queries”, “commit” endpoints, an MCP tool could perform an entire “run database migration” task. Such composite tools can call multiple underlying API endpoints internally. (This typically goes beyond what pure codegen can do automatically – it’s a hybrid approach requiring some manual definition ￼.)
	•	Simplifying Inputs & Outputs: Agents often have limited context length and may not handle very complex JSON or many optional parameters well ￼. We should favor endpoints with simpler request schemas or provide sane defaults. If an API response is very verbose or paginated, consider having the MCP server post-process or summarize it before returning to the agent. For example, if an endpoint returns a full month of data but the agent only asked for today’s info, the wrapper could trim the response. This avoids wasting the agent’s token budget on irrelevant data (a real problem observed when naive MCP mappings returned huge JSON payloads, quickly hitting token limits) ￼.
	•	Clear Usage Instructions: The OpenAPI summary/description for an endpoint is written for developers (“Gets a list of user’s repositories”). For agents, we should rewrite descriptions to emphasize when and why to use the tool, not just what it does ￼. The documentation presented to the agent should guide its decision: e.g., “Use this to retrieve a user’s repository list when you need to find a repo by name.” Including examples of usage, typical workflows, and tips about parameters can greatly help the agent ￼. In other words, the MCP tool documentation should be educational, not just a mirror of the API docs.
	•	Tool Naming and Granularity: Rename or alias tools with concise, action-oriented names that make sense to the agent. For instance, an API endpoint GET /v1/users/{id}/relationships could be exposed as a tool named getUserFollowers with description “Find who follows a given user.” Consistent, descriptive naming helps the agent pick the right tool. Also avoid presenting multiple tools that do almost the same thing – prefer one tool with a parameter switch if possible.

By following these steps, we treat the OpenAPI spec as a starting point rather than the final design. As one expert notes, a direct 1:1 MCP proxy of a REST API is “broken by design” for LLM agents ￼. A better approach is to auto-generate an initial set of tools from the spec, then curate and augment them for agent use ￼. This may involve editing the OpenAPI document itself (e.g. removing endpoints, tweaking descriptions) or modifying the codegen output. The goal is an MCP server that exposes a lean, meaningful interface where each tool is easy to understand and tied to a plausible agent intent.

Automatic Authentication Handling

Most OpenAPI specs include a securitySchemes section describing how the API is secured – for example, API keys, OAuth2 flows, or HTTP Bearer tokens ￼. This is critical for generating a working MCP wrapper, since the server must handle auth when calling the real API. A robust code generation pipeline should detect these schemes and implement them automatically and securely:
	•	API Key Injection: For APIs using static keys or tokens (e.g. an API key in a header or query param), the generator can incorporate this by configuration. For instance, the OpenAPI might mark a header "X-API-Key" or a query parameter as an API key. The Rust server can be generated to read a secret (from an environment variable, config file, etc.) and inject it into outgoing requests transparently ￼. A tool like openapi-mcp supports flags like --api-key-name and --api-key-loc to map a provided key into the request ￼. Crucially, the MCP server should not expose the key to the agent or require the agent to supply it – it can manage the credential internally. This keeps sensitive tokens hidden from the AI client ￼. In Rust, one could leverage frameworks (e.g. using an Axum extractor or Tower middleware) to insert the key on every request to matching paths. Storing the key in a secure manner (environment variable, Vault, etc.) and only using it in the outbound HTTP client ensures security.
	•	OAuth2 Flows: Many large APIs (e.g. Twitter, LinkedIn, Facebook) use OAuth2. Their OpenAPI specs often detail available flows (authorization code, client credentials, etc.) and required scopes ￼. Handling OAuth2 is more complex: it may require obtaining an access token from an auth server and refreshing it. The code generator can detect an OAuth2 scheme and ideally scaffold the needed support. In practice, there are two approaches:
	•	Simplified (Token Forwarding): Assume the operator of the MCP server will provide a valid access token or client credentials out-of-band. The server then simply attaches this token to API requests (e.g. in the Authorization: Bearer <token> header). This is similar to the “forwarding” approach: for example, a config might specify that the MCP server should forward any Authorization header it receives to the upstream API ￼. This enables multi-user scenarios (each agent client could present their own token for the service). However, it requires the token to be obtained beforehand (through a separate login or dev token).
	•	Automated (Token Management): The MCP server itself can be coded to perform the OAuth2 flow. For instance, if using a client credentials grant (common for server-to-server integrations), the server could use a Rust OAuth2 crate to fetch a token from the tokenUrl given in the spec and cache it. Tools like oauth2 (Rust crate) or openidconnect can handle the flow securely. In a user-centric flow (authorization code), it’s trickier since human interaction is needed – not usually suitable for fully automated agents. In most cases for agent tools, a pre-generated token or API key is used rather than having the agent go through login. So, the codegen might simply include placeholders for the developer to insert credentials.
	•	Multiple Auth Methods: Some specs present multiple auth options. For example, Twitter’s OpenAPI lists both OAuth2 user tokens and a simpler Bearer token (for app-only auth) ￼ ￼. A generated MCP server could support either mode. It might default to one (e.g. prefer OAuth2 Bearer) but allow configuration. In Rust, this could mean having an enum or strategy pattern for auth injected into the HTTP client calls. The codegen would check the spec’s security requirement for each operation: if an endpoint requires OAuth2 with certain scopes, the server might verify the configured token has those scopes (if known).
	•	Secure Storage and Usage: Regardless of auth type, keys and secrets should never be hard-coded. The Rust server can load them from environment at startup (using dotenv or std::env) or from a secrets manager. By managing auth centrally, the MCP server acts as a safe broker. The AI agent would simply call the MCP endpoint without needing to know any credentials. This is a major advantage of an MCP wrapper – it decouples tool usage from credential knowledge. For instance, one open-source MCP bridge injects API keys from env and explicitly “keeps API keys hidden from the end MCP client (e.g., the AI assistant)” ￼.
	•	Error Handling & Refresh: Implement logic to handle auth errors – e.g. if a token expired, the server could attempt a refresh (if refresh token or client creds available), or return an error tool response that the agent can understand (maybe suggesting the need for re-auth). In an agent context, you might design the server to output a structured error like {"error": "unauthorized", "message": "API token expired or invalid"} so that the orchestrator can catch it. Logging such events server-side is also important for maintainers.

In summary, OpenAPI specs do provide the necessary info to automate much of the auth. The reliability of this info is generally good for well-known APIs – e.g. specs from providers like Stripe or Twitter clearly define their auth schemes. Some community or third-party specs may omit details, so a quick verification against the docs is wise. Automatic detection of auth scheme from the spec (via the components.securitySchemes and operation security fields) is a key step. After that, generating the appropriate request signing code or middleware ensures the MCP server can communicate with the API securely on behalf of the agent.

Auth Info in OpenAPI Docs – How Reliable Is It?

In practice, how consistent and reliable are OpenAPI docs regarding authentication? This varies across API providers:
	•	Well-Defined Specs: Many modern APIs publish OpenAPI 3.0 specs that include auth details. For example, Twitter’s official OpenAPI JSON includes an OAuth2UserToken scheme with explicit auth and token URLs, scopes, etc. ￼. Stripe’s OpenAPI spec similarly defines its API key header. When such info is present, it can be trusted as the source of truth for implementing auth. The code generator should leverage it fully (for instance, identifying the header name or OAuth flow automatically).
	•	Partial or Missing Info: Not all OpenAPI specs are comprehensive. Some may list a security scheme but not detail the scopes, or they might leave out the security field on operations. Smaller APIs might not use the OpenAPI securityScheme at all, instead just mentioning in descriptions “you must include an API key”. For example, a third-party spec for a weather API might not formalize the API key location. In these cases, the generator might need hints (perhaps manual config) – e.g., an option to treat a certain parameter as a key. The developer might have to augment the spec or supply flags (like --api-key-name as in openapi-mcp) ￼.
	•	Variations Across Platforms: Common patterns exist (API key in header or query; OAuth2 bearer tokens), but each platform has quirks. E.g. some APIs accept a query ?api_key= while others use a header Authorization: Bearer. OpenAPI allows describing both; it’s usually accurate if the spec comes from the provider. When generating wrappers for social media APIs, note that:
	•	Twitter/X: The official spec defines OAuth2 and bearer token usage. It requires keys and tokens that must be obtained via the Twitter developer portal. The spec is reliable, but since Twitter’s API access policies changed in 2023, you must ensure your app has the right tier for the endpoints in use. The OpenAPI won’t tell you if you’re allowed to use an endpoint (that depends on your account tier).
	•	Facebook (Meta) Graph API: Facebook’s Graph API does not provide an official OpenAPI spec ￼. Authentication (via OAuth2 user tokens or app tokens) is documented in prose on their developer site. Thus, any codegen would rely on a manually created spec or third-party definitions (which might not be fully up-to-date). Auth info reliability here is as good as the spec source – caution is needed. Typically, Graph API calls include a token (and often an appsecret proof) as query params. An MCP wrapper would likely need custom logic to handle the OAuth flow (e.g. get a long-lived Page Access Token manually and configure it in the server).
	•	LinkedIn API: LinkedIn’s APIs are likewise not openly documented via OpenAPI. They use OAuth2 (with user tokens) and have strict permissions. OpenAPI info might only be available for certain parts (e.g. their Marketing API) or through community specs. So automatic detection isn’t straightforward – one might have to code the auth handling by reading LinkedIn’s docs (which say, for example, use OAuth2 with certain scopes and a 24h token life). It’s worth noting LinkedIn’s API is limited to approved uses; the spec (if found) might not highlight those policy constraints.

Overall, when the OpenAPI is official and recent, authentication info is usually trustworthy and consistent. The codegen should parse it and implement accordingly. When dealing with unofficial or incomplete specs, developers should be prepared to fill in the gaps. Always double-check if the spec’s described auth matches reality (e.g., some older Swagger files might not reflect a new OAuth requirement added later).

Finally, beyond the spec, consider the agent’s perspective on auth: an agent will not handle interactive logins, so the wrapper must handle auth behind the scenes. Any user-specific tokens likely need to be obtained outside the MCP server (perhaps via a one-time manual OAuth consent by the user, then stored). The OpenAPI spec won’t manage that hand-off, but it at least informs what kind of token and scope are needed.

Wrapping Social Media APIs (Facebook, LinkedIn, Twitter/X) in MCP Servers

Creating MCP wrappers for social media APIs can greatly empower agents (imagine an AI agent that can post on Twitter or read LinkedIn data on your behalf). It is technically feasible, but there are noteworthy challenges:
	•	OpenAPI Availability: As mentioned, not all social platforms provide OpenAPI specs. Twitter does (for v2 API), which simplifies generation ￼. In contrast, Meta’s Graph API and LinkedIn’s API lack official OpenAPI definitions ￼. Developers may use community-crafted specs or generate one by scraping documentation. This adds upfront effort and potential inaccuracies. If an official spec is absent, one approach is to hand-select endpoints you need and write a minimal OpenAPI YAML for them (just enough to generate code or inform the agent). This can still speed up development compared to coding from scratch.
	•	Authentication Complexity: Social APIs use user-centric auth. For example, to call the Twitter API on behalf of a user, you must have that user’s OAuth token (with the needed scopes). Your MCP server therefore needs access to user tokens. In a multi-agent system, if each agent corresponds to a human user, you might need a system for users to connect their accounts and store their tokens (similar to how third-party apps like Buffer work). Handling multi-user auth goes beyond basic codegen – it requires a token management system. Alternatively, for agent use-cases that don’t require user context, you might use app-level access (e.g. Twitter app-only bearer token for limited endpoints). Keep in mind that certain actions (posting, reading DMs, etc.) absolutely require user auth. The MCP server must be designed to either work with a single service account (if appropriate) or be deployed per user to isolate credentials. Projects like the MCP-OpenAPI bridge allow “forwarding the appropriate authorization headers” so multiple users can share one MCP service but pass their own creds ￼.
	•	Rate Limits and Usage Policies: Social media APIs tend to have strict rate limits. An AI agent, if not careful, could exhaust those limits quickly by polling or retrieving large data repeatedly. For instance, as of 2023 Twitter’s free tier only allows a very low volume of calls (on the order of 50 requests per day or 1,500 tweets/month) ￼, and their basic paid tier ($100/month) is needed for anything beyond minimal use ￼. Facebook and LinkedIn also have per-user and global app rate caps (LinkedIn sets daily call limits per developer app, e.g. 100,000 calls/day for certain APIs by policy). An MCP server should account for this:
	•	Rate limiting strategy: The wrapper could enforce delays or limits, returning an error or warning tool output if the agent exceeds safe call rates. This prevents hitting hard API limits (which might otherwise ban or suspend the app). In Rust, one might use middleware (Tower layers) or counters in state to reject or queue calls when near limits.
	•	Graceful degradation: If an agent tries to fetch a lot of data (e.g. all tweets of a user with millions of followers), consider whether the MCP server should chunk the requests or refuse. The agent may not anticipate the cost of certain calls.
	•	Policy compliance: Some social APIs forbid certain uses (for example, LinkedIn disallows writing content on personal profiles via API for most apps, or Facebook might require review for posting to groups). The MCP server, being for internal use, doesn’t bypass these rules. It should stick to allowed endpoints and perhaps encode in the agent instructions what is possible. E.g., if posting to LinkedIn isn’t permitted for your app, don’t expose a postToLinkedIn tool at all.
	•	Maintenance and API Changes: Social platforms are notorious for changing their APIs or terms. A generated wrapper might break as endpoints are versioned or retired. OpenAPI specs (if available) should be kept updated. With Twitter/X, note that the API has gone through changes (v1.1 vs v2, new “X” branding) – ensure you use the latest spec. For Facebook, versioned Graph API changes could require updating your custom spec frequently. A good practice is to design the MCP server to be easily updatable – possibly script the regeneration process so you can re-run it on a new spec file when the API updates.
	•	Example – Twitter MCP: To illustrate viability, suppose we build a Twitter MCP server. We obtain the official openapi.json for Twitter API v2. Using codegen, we create endpoints for a handful of tools: e.g. searchRecentTweets(query), createTweet(text), getUserByUsername(username). We configure the server with our Twitter App’s credentials (API key/secret and bearer token or OAuth2 token). The MCP server handles auth by attaching the bearer token on requests. An agent can now call searchRecentTweets tool with a query – the wrapper calls Twitter’s /2/tweets/search/recent and returns the JSON results. This is viable, but only within the usage limits. If the agent tries to call it in a loop or retrieve huge timelines, it will hit the free tier cap quickly ￼. For production, one would likely need a paid tier for higher limits ￼, or cache results aggressively. Similar logic applies to a Facebook MCP (e.g. an agent tool to getFacebookPagePosts(page_id) that uses the Graph API) – it can work, but you must supply a valid token (perhaps a long-lived Page token) and respect any platform quotas.
	•	Documentation and Agent Guidance: When wrapping these APIs, spend effort on the tool descriptions for the agent. Indicate any important limitations. For example: “searchRecentTweets: Note – this tool can only return up to 100 tweets and may fail if used too frequently due to rate limits.” Such hints could deter the agent from calling too aggressively, or at least make it aware of possible failures.

In summary, wrapping social media APIs is possible but needs caution. Lack of official OpenAPI specs (in some cases) means more manual work. Authentication is the biggest hurdle – ensure a secure flow for token provisioning. And given the often severe rate limits and policies, these MCP servers might be better suited for controlled or low-volume agent tasks, rather than unlimited exploration. If the use-case is agent automation of one’s own accounts (personal or within a company), the approach is quite viable. But if one imagined a general-purpose agent that can access arbitrary user’s social data, it becomes far more complicated (due to needing user-by-user auth consent and running into platform policy issues).

Architecture and Tooling for Rust Code Generation

Building an MCP server in Rust can be tackled in two ways: dynamic runtime generation or compile-time code generation. The Rust ecosystem offers tools for both, and the best approach may be a hybrid.

1. Dynamic Approach (Runtime-driven):
In this model, you write a generic Rust server that can load an OpenAPI spec (from a file or URL) at startup and then mount endpoints accordingly. The server essentially acts as a proxy that doesn’t know the API specifics at compile time – it reads the spec, perhaps filters it, and then uses a generic handler to forward requests. Key components for this approach:
	•	Parsing OpenAPI: Use a crate like openapiv3 (for OpenAPI 3.x) to parse the spec YAML/JSON into Rust structures. This gives you a programmatic view of all paths, operations, parameters, and schemas.
	•	Routing: Using a web framework such as Axum or Actix-web, you can programmatically register routes. For each path+method in the spec (that you choose to expose), set up an endpoint in the Rust server. For example, if the spec has GET /v1/customers, you create an Axum route GET /api/stripe/v1/customers (perhaps namespaced under the service). The handler for all such routes can be a generic function that looks up the target path and method.
	•	Forwarding Logic: The handler will take the incoming MCP tool call (which includes JSON payload or query params from the agent), map it to an HTTP request to the real API. This can be done using Reqwest (an async HTTP client) to call the external URL (spec’s server + path). You’d construct the URL, inject query parameters, headers, and body as per the spec’s parameter definitions. This is akin to a reverse proxy guided by the OpenAPI. Authentication can be applied here (e.g. add Bearer token header via middleware or in the handler before sending the request).
	•	Type Conversion: The OpenAPI spec defines schemas for requests/responses. A dynamic approach might simply treat bodies as opaque serde_json::Value without generating concrete Rust types. This is fine – you can accept Value from the agent and forward it, and pipe back the JSON Value from the external API to the agent. If you want more type safety or to validate the agent’s inputs against the schema, you could integrate a JSON Schema validator using the spec’s schemas. There are crates like jsonschema or you could leverage schemars to generate Rust types and then use Serde to validate.
	•	Selective exposure: As discussed, you may not want every path. You can match paths against a regex or list (perhaps provided in a config file) and only mount those ￼. This prevents the agent from even seeing or calling unneeded endpoints.
	•	Performance: Parsing a large spec (hundreds of endpoints) can be slow at startup (multi-megabyte specs). Caching the parsed representation or using a “slim” spec (with only needed parts) can mitigate cold starts ￼. Once running, the overhead per request is mostly the network call to the API – the Rust server can handle many concurrent requests with async I/O.

The dynamic method is flexible – one binary can serve many different API specs (even simultaneously, by mounting each under a different base path as seen in some examples ￼). It’s great for quickly supporting new APIs by configuration. The trade-off is that you don’t get compile-time type checks for each endpoint, and adding custom logic for specific endpoints (like combining calls or special formatting) can get messy if done dynamically.

2. Static Code Generation (Compile-time):
This involves converting the OpenAPI spec into Rust source code (models, trait interfaces, function stubs) ahead of time. Tools and techniques include:
	•	OpenAPI Generator (CLI): The official OpenAPI Generator supports a rust-server target which generates a Rust project (using Hyper and Tower) with an interface for each operation ￼. Similarly, it has a rust client generator. One could run this generator on a given spec, then modify the output to suit MCP needs (e.g. editing descriptions, merging functions, etc.). This is a more manual process but yields fully concrete code. The rust-server output gives you a stub for each endpoint – you can implement each by calling the external API (perhaps using the generated client or with Reqwest).
	•	Rust Crates like Paperclip: Paperclip is a crate that can generate Rust types from OpenAPI and even some server code, though it was initially more for documenting existing Rust APIs. It’s an option if you prefer a Rust-native solution for codegen, but it might require the spec to be Swagger v2 in some cases ￼.
	•	Progenitor (for clients): Tools like Progenitor generate Rust API client libraries from an OpenAPI spec ￼. Instead of writing the HTTP calls yourself, you could use Progenitor to generate a strongly-typed client for the target API (with methods corresponding to each endpoint). Then, in your MCP server code, simply call those client methods. This can speed up development and ensure parity with the API’s spec. For example, if the spec has an operation “createTweet”, the generated client might have client.create_tweet(params...). Your MCP handler can call that and get a Rust struct response. Progenitor is async and works with Reqwest under the hood ￼ ￼.
	•	Function Macros: Progenitor even allows a macro generate_api!("path/to/spec.json") that will autogenerate the client code into your binary at compile time ￼. You could combine this with an MCP server framework. Essentially, this gives you compile-time assurance (if the spec changes, you’ll see compile errors for removed endpoints, etc.).

Using static generation yields a more structured codebase. It’s easier to inject custom logic into specific endpoints if needed. You can also tailor the documentation strings of each tool function easily – since they may come from the OpenAPI descriptions, you can edit them in the spec or in the generated code (or via templates if using a custom generator pipeline). A suggested workflow is: generate the initial server, then apply the “prune and polish” steps. For example, generate all 50 endpoints, but then only keep 10 that make sense, and manually adjust their handlers and docs. This matches the recommended hybrid approach ￼ – codegen saves time, but you refine it for the final product.

3. Rust Framework Considerations:
Regardless of dynamic or static, you’ll choose a web framework to implement the server:
	•	Axum (with Hyper under the hood) is very ergonomic for defining routes and supports WebSocket or SSE for streaming results if needed. MCP often uses streaming responses (Server-Sent Events or similar) to continuously send output tokens. For short API calls this isn’t crucial, but if the agent expects an SSE endpoint, Axum’s Sse extractors can be used.
	•	Actix-web is another popular framework, also async and performant. It could be used similarly. Actix has good support for JSON, and there are crates to help integrate OpenAPI with Actix (like Paperclip’s actix plugin).
	•	Tower and lower-level Hyper can be used if you rely on the OpenAPI Generator’s output (since it uses Tower Service traits). This is more low-level but yields fine control.

For making outbound API calls, Reqwest is the go-to HTTP client in Rust (async and simple). It will handle TLS, JSON encoding/decoding (via Serde) etc. If the API uses HTTP/2 or has streaming endpoints, Reqwest supports that too.

4. Example: Weather API Wrapper
To illustrate an end-to-end approach, imagine wrapping a simple Weather API (like OpenWeatherMap, which requires an API key in queries). You could write a small Rust MCP server that on startup reads the OpenWeatherMap OpenAPI spec. Suppose you only care about the /weather endpoint (current weather by city). The server registers a tool getWeather(city: String) for agents. Internally, when the agent calls this, the handler takes the city name, makes a GET request to /data/2.5/weather?q={city}&appid={API_KEY} (the API key is appended from config). The response JSON is returned directly to the agent. The OpenAPI spec would have told you the base URL and that appid is a required query param – your code injects it. In the config, you map perhaps a header X-Open-Weather-App-Id to the appid param ￼ so that you don’t log the raw key. This is a straightforward wrapper that was quick to set up, and the agent now has a high-level getWeather tool instead of dealing with raw HTTP. Similar patterns scale up to more complex APIs.

5. Useful Crates and Tools Recap:
	•	openapiv3 – Parse OpenAPI specs in Rust (supports v3.x). For v2 (Swagger) there’s swagger or use the same crate if it can convert.
	•	schemars / serde_json – Handle JSON Schema or dynamic JSON values for validating agent inputs against the spec.
	•	Reqwest – HTTP client for external API calls (with TLS, JSON, auth support).
	•	OAuth2 – Rust crate (on crates.io) for performing OAuth flows (if needed for automatic token retrieval).
	•	Axum / Actix-web – Web frameworks to implement the MCP HTTP server endpoints.
	•	tokio – Async runtime, used by the above for concurrency.
	•	OpenAPI Generator (external CLI or Maven plugin) – if you prefer to generate a Rust project from the spec. It can save time writing boilerplate.
	•	Progenitor (by Oxide Computer) – for generating a Rust client library from an OpenAPI spec, which you can integrate into your server ￼.
	•	paperclip – for codegen and documentation in Rust (though still maturing, can generate some code from Swagger v2) ￼.

Architecturally, these MCP servers are essentially stateless proxies with some domain knowledge. They accept structured requests from an AI agent (often via JSON over HTTP or an SSE stream), call the real API, and return the result. Rust’s performance and safety make it a good choice: it can handle many concurrent agent requests, and using strong types (when available) helps ensure the agent’s commands translate correctly to API calls. Just be mindful of logging and error messages – avoid leaking secrets, and consider sanitizing any data that goes back to the agent if needed.

Conclusion

Automatically generating Rust-based MCP servers from OpenAPI specs can greatly accelerate connecting AI agents to real-world services. The OpenAPI spec provides a blueprint of the API’s capabilities, but bridging the gap to agent-friendly design requires thoughtful post-processing: curating the toolset, clarifying usage, and sometimes enhancing endpoints for the agent’s convenience. Authentication should be handled behind the scenes by the MCP server, leveraging the spec’s info to inject API keys or manage OAuth tokens securely without burdening the agent ￼.

Our exploration shows that common patterns (API keys, bearer tokens, OAuth2) can indeed be auto-detected and implemented – many tools already do this in other languages ￼ ￼. For social media APIs, one must navigate the additional hurdles of missing specs and strict access limits, but it is feasible with careful planning and possibly paying for higher API tiers ￼.

By using the robust Rust ecosystem – from OpenAPI parsing libraries to HTTP clients and web frameworks – developers can create reliable MCP wrappers. These wrappers act as “smart adapters” ￼, translating an AI’s intent into API calls and returning machine-friendly results. When done well, the AI agents in a system like Socialings AI can “plug into” powerful external services through these standardized MCP interfaces, without custom integration code for each new API ￼. The result is a more scalable, modular agent architecture where adding a new tool is as simple as pointing to an OpenAPI spec and hitting build.

Sources:
	•	Lisowski, E. Model Context Protocol (MCP) Explained – Medium, Apr 2025 ￼ ￼
	•	Neon Tech Engineering – Auto-generating MCP Servers from OpenAPI Schemas (Blog), May 2025 ￼ ￼
	•	Ckanthony (GitHub) – openapi-mcp Project (README) ￼ ￼
	•	Conor Branagan (GitHub) – mcp-openapi Project (README) ￼
	•	Twitter API v2 – Official OpenAPI Specification (example auth schemes) ￼
	•	Stack Overflow – Facebook API in machine-readable format (discussion on lack of official spec) ￼
	•	X (Twitter) Developer Community – Free Tier Limits (forum post), 2023 ￼
	•	TechCrunch – Twitter API basic tier $100/month for low usage, Feb 2023 ￼


Here’s a high-level technical review of what we already have, the gaps I see, and a concrete proposal for a config-driven Rust code-gen toolchain that turns any OpenAPI spec into a slim, agent-friendly MCP micro-service with automatic auth and endpoint whitelisting.

⸻

TL;DR summary

Your generate_handlers.rs prototype already proves the core idea: parse an OpenAPI file with openapiv3, template out Axum handlers, and skip “monster” endpoints via a hard-coded exclusion list. To take this from a one-off script to a reusable product we need:
	•	Externalised build-spec (TOML/YAML) to drive what gets generated and how.
	•	Auth middleware generation driven by components.securitySchemes (API-key, OAuth2, bearer) so secrets stay server-side.
	•	Pluggable templates (Handlebars or Tera) to let users drop in their own handler style or rename tools.
	•	Selective path/operation filters via regex or tag list, similar to mcp-openapi’s paths: clause .
	•	A small CLI (cargo run -- gen) built with clap for nice UX .
	•	Optional strong-typed client generation (via Progenitor) so handlers call a safe Rust client instead of ad-hoc reqwest .

The net result: you point the tool at a spec + config file, it spits out a ready-to-compile Rust MCP crate with only the endpoints and docs an agent actually needs.

⸻

1 — What our current generator does well

Feature	Notes
OpenAPI parse	Uses openapiv3 to walk paths/operations .
Axum stub template	Templated handlers compile quickly.
Endpoint exclusion	Giant download_fulllist route was skipped manually.
Minimal runtime deps	Only serde, reqwest, tokio, axum.

Those decisions keep compile times low and make the binary suitable for container-first deployment.

Pain points observed
	1.	Filtering rules are in code, so every new API means editing and recompiling.
	2.	No auth helpers – whoever runs the service has to wire tokens by hand.
	3.	Handler template is locked-in – can’t easily swap for streaming, tracing, etc.
	4.	No spec diff / regen – changing an OpenAPI file doesn’t auto-update Rust models.
	5.	Docs for agents still copy the OpenAPI verbatim – not task-oriented.

⸻

2 — Designing a configurable generator

2.1  Build-spec file (mcpgen.yaml)

service_name: twitter
spec_url: https://api.twitter.com/2/openapi.json
output_dir: ./generated/twitter
include_paths:
  - "^/tweets/search/recent$"
  - "^/tweets$"
exclude_paths:
  - "^/tweets/:id/hidden$"
rename:
  "^/tweets/search/recent$": search_recent_tweets
  "^/tweets$": create_tweet
auth:
  scheme_preference: [oauth2, bearer, apiKey]
  env_vars:
    oauth2_client_id: TW_CLIENT_ID
    oauth2_client_secret: TW_CLIENT_SECRET
    bearer_token: TW_BEARER

	•	include_paths / exclude_paths mirror what mcp-openapi exposes with its paths: regex block  and what devs have requested in OpenAPI-Generator feature requests .
	•	rename lets you give agent-friendly names (search_recent_tweets) instead of RESTy verbs.
	•	auth.scheme_preference tells the generator which securityScheme to wire first; the CLI inspects components.securitySchemes and builds an Axum/Tower middleware that injects the right header/query param automatically .

2.2  Code generation pipeline
	1.	Parse spec (openapiv3).
	2.	Filter & rename operations per config (regex/tag). Regex filtering is documented as a common need in Swagger/NSwag threads .
	3.	Generate strongly-typed client with Progenitor (one Client per spec) .
	4.	Render handlers via Handlebars templates ; each template can read operation metadata and spit out an Axum route that calls the Progenitor client.
	5.	Insert auth layer:
	•	API-key → add header/query from env var.
	•	OAuth2 client-cred → generate a small token-cache struct using oauth2 crate.
	6.	Generate agent-facing tool docs (summary, when-to-use, example). These live in a tools.json so orchestrators (LangChain, Autogen) can read them – same idea as mcp-openapi /tools endpoint .
	7.	Emit Cargo workspace with: server/, client/, schema/ crates – everything ready to cargo build --release.

2.3  CLI UX (mcpgen)

mcpgen init twitter.yaml   # scaffold config
mcpgen check twitter.yaml  # validate filters vs spec
mcpgen generate twitter.yaml
mcpgen serve twitter.yaml  # hot‐reload dev server

Built with clap v4 so you get colorised help, shell completions, and fuzzy suggestions .

⸻

3 — Template packs
	•	axum-basic – JSON in/out, no streaming.
	•	axum-sse – streams external chunks as MCP SSE (model-token-friendly).
	•	actix-middleware – if someone prefers Actix.
Users can ship their own folders under ~/.config/mcpgen/templates/ – OpenAPI-Generator supports custom Mustache templates similarly .

⸻

4 — Handling huge specs & rate-limited APIs
	•	Borrow the slim-openapi trick from mcp-openapi to resolve $refs and write a shrunk spec before code-gen .
	•	For social platforms with missing or partial specs (Meta, LinkedIn) you can hand-craft a minimal YAML, then the same pipeline works.
	•	Inject Tower rate-limit middleware and error mapping so the agent sees a clean "rate_limit_hit" JSON instead of raw 429 .

⸻

5 — Next steps for us
	1.	Share the raw generate_handlers.rs (GitHub’s UI blocked the unauth’d raw fetch; paste it here or gist it so I can comment line-by-line).
	2.	Prototype the config parser (Serde TOML) and the operation filter/rename pass – fastest incremental value.
	3.	Swap your existing string-concat template for Handlebars so future tweaks are one-line edits, not code surgery.
	4.	Decide whether you or open-source community hosts the crate. A polished CLI could grab traction the way mcp-openapi did .

With these pieces, you’ll have a one-command Rust factory for bespoke, agent-savvy MCP bridges – and that’s exactly the tooling gap folks are hunting for in multi-agent land. 🚀🦀

⸻

References
	1.	openapiv3 crate docs – structural model of spec
	2.	openapiv3::OpenAPI fields for security schemes
	3.	Progenitor README – Rust client generation
	4.	clap 4.x – polished CLI parsing
	5.	mcp-openapi README – selective route exposure & header forwarding
	6.	Handlebars-rust – pluggable template engine
	7.	OpenAPI-Generator custom template docs
	8.	OpenAPI-Generator issue on operation exclusion
	9.	StackOverflow answer on excluding paths during code-gen
	10.	NSwag GitHub discussion on selective client generation
	11.	Paperclip project – Rust OpenAPI tooling
	12.	Tower/Actix middleware patterns for auth/rate-limit


💡 Quick verdict: spin up a new, clean OSS repo for the generator itself, then depend on it from the existing FDIC project as a dev-tool / build step.
That gives us a tidy separation of concerns, lets others adopt the generator without dragging in FDIC-specific code, and positions the FDIC repo as a living showcase.

⸻

🥊 Pros & cons

Stand-alone “mcpgen” repo	Keeping it inside FDIC repo
✅ Clear brand & README focused on the generator	🔄 Less initial repo juggling
✅ Easier for outsiders to star/fork & contribute	❌ Newcomers sift through FDIC domains first
✅ Semantic versioning via cargo publish (crate)	❌ Hard to publish as crate without cruft
✅ Multiple example fixtures (FDIC, Stripe, etc.)	❌ Every example pollutes one repo
✅ License can be liberal (MIT/Apache)	❌ Tied to whatever license FDIC code uses
✅ Future plug-ins / templates can live here	❌ Generator changes risk breaking FDIC CI

Verdict: new repo wins for maintainability, adoption, and your street-cred. 🏆

⸻

🔨  Minimal repo layout

mcpgen/
├── crates/
│   ├── mcpgen-cli/        # clap-powered binary
│   ├── mcpgen-core/       # parsing, filters, auth middleware gen
│   └── templates/         # handlebars packs
├── examples/
│   ├── fdic_bankfind/     # points to your current repo as submodule or script
│   ├── twitter/           # sample spec + config
│   └── openweather/
├── README.md
├── LICENSE
└── CHANGELOG.md

Publish mcpgen-core and mcpgen-cli on crates.io; users can embed or cargo install.

⸻

🚀  Migration steps
	1.	Create the repo (github.com/<org>/mcpgen) with MIT/Apache-2.0 dual license.
	2.	Copy & trim generate_handlers.rs into mcpgen-core/src/lib.rs, splitting into modules (parser, filter, auth, render).
	3.	Write the Clap wrapper in mcpgen-cli.
	4.	Move Tera templates into templates/axum-basic/*.tera.
	5.	Add FDIC fixture under examples/, with a tiny script showing mcpgen generate fdic.toml.
	6.	Refactor the FDIC repo:
	•	Remove generator code.
	•	Add mcpgen to [build-dependencies] and invoke it from build.rs or keep a scripts/generate.sh that runs the CLI.
	7.	CI: GitHub Actions for cargo test, cargo fmt, and example generation smoke-test.
	8.	Publish v0.1.0 of both crates.
	9.	Announce the project—people building LangChain, Autogen, or OpenAI Tool wrappers in Rust will care.

⸻

📝  Next move for us

If you say “Let’s launch mcpgen,” I’ll:
	•	scaffold the new repo structure,
	•	commit an initial Cargo.toml for mcpgen-core,
	•	port the CLI skeleton,
	•	push a PR-style patch (using oboe.edit_file on your behalf).

We can then iterate on filters, auth, and docs.

Sound like the plan you want? If so, give me the green light and I’ll fire up the scaffolding. 🦀🚀