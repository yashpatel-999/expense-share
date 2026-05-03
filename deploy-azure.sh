#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
RESOURCE_GROUP="expense-share-rg"
LOCATION="eastus"
ACR_NAME="expenseshareacr"
CONTAINER_ENV="expense-share-env"

echo -e "${YELLOW}=== Expense Share Azure Deployment ===${NC}"
echo ""

# Step 1: Login
echo -e "${YELLOW}Step 1: Logging into Azure...${NC}"
az login

# Step 2: Create resource group
echo -e "${YELLOW}Step 2: Creating resource group...${NC}"
az group create --name $RESOURCE_GROUP --location $LOCATION

# Step 3: Create ACR
echo -e "${YELLOW}Step 3: Creating Azure Container Registry...${NC}"
az acr create --resource-group $RESOURCE_GROUP --name $ACR_NAME --sku Basic

# Step 4: Login to ACR
echo -e "${YELLOW}Step 4: Logging into ACR...${NC}"
az acr login --name $ACR_NAME

# Step 5: Build and push images
echo -e "${YELLOW}Step 5: Building and pushing Docker images...${NC}"

echo "Building backend image..."
docker build -t $ACR_NAME.azurecr.io/expenses-backend:latest ./expenses_backend
docker push $ACR_NAME.azurecr.io/expenses-backend:latest

echo "Building frontend image..."
docker build -t $ACR_NAME.azurecr.io/expenses-frontend:latest ./expenses_frontend
docker push $ACR_NAME.azurecr.io/expenses-frontend:latest

# Step 6: Create Container App Environment
echo -e "${YELLOW}Step 6: Creating Container App Environment...${NC}"
az containerapp env create --name $CONTAINER_ENV \
  --resource-group $RESOURCE_GROUP \
  --location $LOCATION

# Step 7: Get ACR credentials
echo -e "${YELLOW}Step 7: Getting ACR credentials...${NC}"
ACR_USERNAME=$(az acr credential show --name $ACR_NAME --query username -o tsv)
ACR_PASSWORD=$(az acr credential show --name $ACR_NAME --query passwords[0].value -o tsv)

# Step 8: Get Database URL
echo -e "${YELLOW}Step 8: Database Configuration${NC}"
echo "Your PostgreSQL database is already hosted externally."
echo "Enter your database connection string:"
read -p "DATABASE_URL: " DATABASE_URL

# Step 9: Get JWT Secret
echo -e "${YELLOW}Step 9: Security Configuration${NC}"
read -sp "Enter JWT secret: " JWT_SECRET
echo ""

# Step 10: Deploy Backend
echo -e "${YELLOW}Step 10: Deploying backend...${NC}"
az containerapp create \
  --name expenses-backend \
  --resource-group $RESOURCE_GROUP \
  --environment $CONTAINER_ENV \
  --image $ACR_NAME.azurecr.io/expenses-backend:latest \
  --registry-server $ACR_NAME.azurecr.io \
  --registry-username $ACR_USERNAME \
  --registry-password $ACR_PASSWORD \
  --target-port 8080 \
  --ingress external \
  --env-vars \
    DATABASE_URL="$DATABASE_URL" \
    JWT_SECRET="$JWT_SECRET" \
    RUST_LOG="info" \
  --cpu 0.5 \
  --memory 1Gi

# Step 11: Get Backend URL
echo -e "${YELLOW}Step 11: Getting backend URL...${NC}"
BACKEND_URL=$(az containerapp show --resource-group $RESOURCE_GROUP \
  --name expenses-backend \
  --query properties.configuration.ingress.fqdn -o tsv)

echo -e "${GREEN}Backend URL: https://$BACKEND_URL${NC}"

# Step 12: Deploy Frontend
echo -e "${YELLOW}Step 12: Deploying frontend...${NC}"
az containerapp create \
  --name expenses-frontend \
  --resource-group $RESOURCE_GROUP \
  --environment $CONTAINER_ENV \
  --image $ACR_NAME.azurecr.io/expenses-frontend:latest \
  --registry-server $ACR_NAME.azurecr.io \
  --registry-username $ACR_USERNAME \
  --registry-password $ACR_PASSWORD \
  --target-port 80 \
  --ingress external \
  --env-vars \
    API_URL="https://$BACKEND_URL/api" \
  --cpu 0.5 \
  --memory 1Gi

# Step 13: Get Frontend URL
echo -e "${YELLOW}Step 13: Getting frontend URL...${NC}"
FRONTEND_URL=$(az containerapp show --resource-group $RESOURCE_GROUP \
  --name expenses-frontend \
  --query properties.configuration.ingress.fqdn -o tsv)

echo -e "${GREEN}Frontend URL: https://$FRONTEND_URL${NC}"

echo ""
echo -e "${GREEN}=== Deployment Complete! ===${NC}"
echo -e "${GREEN}Frontend: https://$FRONTEND_URL${NC}"
echo -e "${GREEN}Backend: https://$BACKEND_URL${NC}"
echo ""
echo "Next steps:"
echo "1. Run database migrations if needed"
echo "2. Test the application"
