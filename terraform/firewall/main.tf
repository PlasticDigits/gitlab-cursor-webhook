terraform {
  required_providers {
    hcloud = {
      source  = "hetznercloud/hcloud"
      version = "~> 1.49"
    }
  }
}

variable "name" {
  type    = string
  default = "gch-agent-firewall"
}

variable "admin_ssh_cidr" {
  type        = string
  description = "Optional CIDR for inbound SSH (debugging). Empty = no inbound SSH."
  default     = ""
}

resource "hcloud_firewall" "gch" {
  name = var.name

  dynamic "rule" {
    for_each = var.admin_ssh_cidr != "" ? [1] : []
    content {
      direction  = "in"
      protocol   = "tcp"
      port       = "22"
      source_ips = [var.admin_ssh_cidr]
    }
  }

  rule {
    direction       = "out"
    protocol        = "tcp"
    port            = "443"
    destination_ips = ["0.0.0.0/0", "::/0"]
  }

  rule {
    direction       = "out"
    protocol        = "tcp"
    port            = "80"
    destination_ips = ["0.0.0.0/0", "::/0"]
  }

  rule {
    direction       = "out"
    protocol        = "tcp"
    port            = "22"
    destination_ips = ["0.0.0.0/0", "::/0"]
  }

  rule {
    direction       = "out"
    protocol        = "tcp"
    port            = "53"
    destination_ips = ["0.0.0.0/0", "::/0"]
  }

  rule {
    direction       = "out"
    protocol        = "udp"
    port            = "53"
    destination_ips = ["0.0.0.0/0", "::/0"]
  }
}

output "firewall_id" {
  value = hcloud_firewall.gch.id
}
