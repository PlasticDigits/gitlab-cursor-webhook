terraform {
  required_providers {
    hcloud = {
      source  = "hetznercloud/hcloud"
      version = "~> 1.49"
    }
  }
}

variable "job_id" {
  type = string
}

variable "server_type" {
  type    = string
  default = "cx33"
}

variable "snapshot_id" {
  type = string
}

variable "location" {
  type    = string
  default = "nbg1"
}

variable "firewall_id" {
  type = string
}

variable "ssh_key_ids" {
  type    = list(string)
  default = []
}

variable "cloud_init" {
  type = string
}

variable "label_tag" {
  type = string
}

variable "label_project" {
  type = string
}

variable "label_iid" {
  type = string
}

variable "label_created" {
  type = string
}

resource "hcloud_server" "agent" {
  name        = "gch-${var.job_id}"
  server_type = var.server_type
  image       = var.snapshot_id
  location    = var.location
  firewall_ids = [var.firewall_id]
  ssh_keys    = var.ssh_key_ids
  user_data   = var.cloud_init

  labels = {
    gch_job_id     = var.job_id
    gch_tag        = var.label_tag
    gch_project    = replace(var.label_project, "/", "-")
    gch_iid        = var.label_iid
    gch_created_at = var.label_created
  }
}

output "server_id" {
  value = hcloud_server.agent.id
}

output "server_ipv4" {
  value = hcloud_server.agent.ipv4_address
}
