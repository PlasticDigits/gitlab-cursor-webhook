#cloud-config
write_files:
  - path: /etc/gch/job.env
    permissions: '0600'
    owner: agent:agent
    defer: true
    content: |
      GCH_JOB_ID=${job_id}
      GCH_CONTROLLER_URL=${controller_url}
      JOB_RUNTIME_TOKEN=${runtime_token}
      CURSOR_API_KEY=${cursor_api_key}
      GITLAB_TOKEN=${gitlab_token}
runcmd:
  - [bash, -lc, "set -euo pipefail; base='https://gitlab.com/plasticdigits/gitlab-cursor-webhook/-/raw/main/scripts'; curl -fsSL \"$base/gch-cloud-init-runner.sh\" -o /home/agent/gch-cloud-init-runner.sh || true; curl -fsSL \"$base/gch-agent-idle-wrap.py\" -o /home/agent/gch-agent-idle-wrap.py || true; chown agent:agent /home/agent/gch-cloud-init-runner.sh /home/agent/gch-agent-idle-wrap.py 2>/dev/null || true; chmod 755 /home/agent/gch-cloud-init-runner.sh /home/agent/gch-agent-idle-wrap.py 2>/dev/null || true"]
  - [bash, -lc, "su - agent -c 'bash ${cloud_init_script}'"]
