### RPC
The Filament Hub Core API provides REST endpoints for interacting with campaign-related functionality. The primary endpoint `/campaigns/{campaign_id}` allows clients to retrieve detailed information about a specific campaign by providing its unique identifier.

TODO: Link to API Documentation

Currently, the API supports fetching individual campaign details which includes the campaigner's address. The endpoint returns a 200 status code with campaign data on success, or a 404 status code if the specified campaign is not found. This API serves as a fundamental interface for applications to query campaign state from the Filament Hub.

The endpoint returns a Campaign object containing:
- `id`: A unique identifier for the campaign (uint64)
- `campaigner`: The address of the campaign creator
- `phase`: Current state of the campaign (one of: Draft, Init, Criteria, Publish, Indexing, Distribution, Settle, Settled, Canceled, or Rejected)
- `title`: Campaign name/title
- `description`: Detailed campaign description
- `criteria`: Distribution criteria specifications
- `evictions`: List of evicted delegates
- `delegates`: List of participating delegates
- `indexer`: Optional address of the assigned indexer
