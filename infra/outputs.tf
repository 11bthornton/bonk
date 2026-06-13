output "fqdn" {
  value = azurerm_container_app.main.ingress[0].fqdn
}

output "acr_login_server" {
  value = azurerm_container_registry.main.login_server
}
