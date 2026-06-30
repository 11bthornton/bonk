terraform {
  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~> 4.0"
    }
  }

  backend "azurerm" {
    resource_group_name  = "hub"
    storage_account_name = "thorntonterraformstate"
    container_name       = "tfstate"
    key                  = "bonk.tfstate"
    use_oidc             = true
  }
}

provider "azurerm" {
  features {}
  subscription_id = var.subscription_id
}

resource "azurerm_resource_group" "main" {
  name     = var.resource_group_name
  location = var.location
}

resource "azurerm_container_registry" "main" {
  name                = var.acr_name
  resource_group_name = azurerm_resource_group.main.name
  location            = azurerm_resource_group.main.location
  sku                 = "Basic"
  admin_enabled       = true
}

resource "azurerm_log_analytics_workspace" "main" {
  name                = "${var.app_name}-logs"
  resource_group_name = azurerm_resource_group.main.name
  location            = azurerm_resource_group.main.location
  sku                 = "PerGB2018"
  retention_in_days   = 30
}

resource "azurerm_container_app_environment" "main" {
  name                       = "${var.app_name}-env"
  resource_group_name        = azurerm_resource_group.main.name
  location                   = azurerm_resource_group.main.location
  log_analytics_workspace_id = azurerm_log_analytics_workspace.main.id
}

data "azurerm_dns_zone" "hub" {
  name                = "ben-thornton.com"
  resource_group_name = "hub"
}

resource "azurerm_dns_txt_record" "bonk_verification" {
  name                = "asuid.bonk"
  zone_name           = data.azurerm_dns_zone.hub.name
  resource_group_name = data.azurerm_dns_zone.hub.resource_group_name
  ttl                 = 300

  record {
    value = azurerm_container_app_environment.main.custom_domain_verification_id
  }
}

resource "azurerm_dns_cname_record" "bonk" {
  name                = "bonk"
  zone_name           = data.azurerm_dns_zone.hub.name
  resource_group_name = data.azurerm_dns_zone.hub.resource_group_name
  ttl                 = 300
  record              = "${var.app_name}.${azurerm_container_app_environment.main.default_domain}"
}

resource "azurerm_container_app_custom_domain" "bonk" {
  name                     = "bonk.ben-thornton.com"
  container_app_id         = azurerm_container_app.main.id
  certificate_binding_type = "Disabled"

  depends_on = [
    azurerm_dns_cname_record.bonk,
    azurerm_dns_txt_record.bonk_verification,
  ]
}

resource "azurerm_container_app" "main" {
  name                         = var.app_name
  resource_group_name          = azurerm_resource_group.main.name
  container_app_environment_id = azurerm_container_app_environment.main.id
  revision_mode                = "Single"

  registry {
    server               = azurerm_container_registry.main.login_server
    username             = azurerm_container_registry.main.admin_username
    password_secret_name = "acr-password"
  }

  secret {
    name  = "acr-password"
    value = azurerm_container_registry.main.admin_password
  }

  template {
    min_replicas = 0
    max_replicas = 1

    container {
      name   = "ballsack"
      image  = "${azurerm_container_registry.main.login_server}/ballsack:latest"
      cpu    = 0.25
      memory = "0.5Gi"

      env {
        name  = "PORT"
        value = "8080"
      }
    }
  }

  ingress {
    external_enabled = true
    target_port      = 8080

    traffic_weight {
      percentage      = 100
      latest_revision = true
    }
  }
}
