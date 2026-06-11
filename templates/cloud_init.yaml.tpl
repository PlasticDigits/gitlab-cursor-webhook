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
  - [bash, -lc, "su - agent -c 'bash ${cloud_init_script}'"]
