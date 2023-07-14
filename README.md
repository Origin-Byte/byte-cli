# Byte CLI

The CLI has the following commands:

#### 1. To configure the collection:

```cargo run --bin byte-cli config-collection <PROJECT_FOLDER>```

#### 2. To configure the upload:

`cargo run --bin byte-cli config-upload <PROJECT_FOLDER>`

#### 3. To deploy the contract on devnet:

`cargo run --bin byte-cli generate-contract <PROJECT_FOLDER>`

`cargo run --bin byte-cli deploy-contract <PROJECT_FOLDER> --skip-generation`
`cargo run --bin byte-cli deploy-contract <PROJECT_FOLDER>`

#### 4. To deploy assets to the storage server:

```cargo run --bin byte-cli deploy-assets <PROJECT_FOLDER>```

#### 5. To Mint NFTs on-chain:

`cargo run --bin byte-cli mint-nfts <PROJECT_FOLDER>`

`cargo run --bin byte-cli generate-contract <PROJECT_FOLDER>`

`cargo run --bin byte-cli check-dependencies`

`cargo run --bin byte-cli deploy-assets tony`

`cargo run --bin byte-cli new-simple pinata_test 1000 100`


## Deploy byte-api to Google Cloud

### 1. Authenticate with Google Cloud Artifact Registry
gcloud auth configure-docker europe-west6-docker.pkg.dev

### 2. Build and push the image
docker build -f Dockerfile -t europe-west6-docker.pkg.dev/originbyte-4ce3a/cloudrun-images/byte-api:latest .
docker push europe-west6-docker.pkg.dev/originbyte-4ce3a/cloudrun-images/byte-api:latest

### 3. Deploy the image to Cloud Run
gcloud run deploy byte-api --image=europe-west6-docker.pkg.dev/originbyte-4ce3a/cloudrun-images/byte-api:latest
select region [20] europe-west6 (Zürich) 


### 4. Service url:
[Title](https://byte-api-gqns6vypmq-oa.a.run.app/swagger-ui/)
(will be private soon)