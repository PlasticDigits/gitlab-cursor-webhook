# GCH Hetzner Firewall

One-time setup for the shared firewall attached to all agent VMs.

## Prerequisites

- [Terraform](https://www.terraform.io/) >= 1.5
- `HCLOUD_TOKEN` environment variable set

## Apply

```bash
cd terraform/firewall
export HCLOUD_TOKEN=your_token
terraform init
terraform apply
```

Note the `firewall_id` output and register it:

```bash
gchconfig setting firewall_id <firewall_id>
# or set GCH_FIREWALL_ID in the controller environment
```

Optional inbound SSH for debugging:

```bash
terraform apply -var='admin_ssh_cidr=203.0.113.10/32'
```
