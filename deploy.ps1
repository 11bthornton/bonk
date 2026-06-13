$ErrorActionPreference = "Stop"

$rg = "bonk-rg"
$acr = "bonkregistry"
$app = "bonk-app"
$env = "bonk-env"
$tenant = "6b80a6f1-7342-4d0f-b8b6-f60a02e2e38d"
$image = "$acr.azurecr.io/ballsack:latest"

Write-Host "Logging in to Azure..." -ForegroundColor Cyan
az login --tenant $tenant
az acr login --name $acr --expose-token | Out-Null

Write-Host "Building image in Azure..." -ForegroundColor Cyan
az acr build --registry $acr --image ballsack:latest .

Write-Host "Updating container app..." -ForegroundColor Cyan
az containerapp update --name $app --resource-group $rg --image $image

$fqdn = az containerapp show --name $app --resource-group $rg --query properties.configuration.ingress.fqdn -o tsv
Write-Host "Deployed to https://$fqdn" -ForegroundColor Green
