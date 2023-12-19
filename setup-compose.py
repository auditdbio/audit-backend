template="""
x-common-variables: &common-variables
  MONGOURI: "mongodb://${MONGO_LOGIN}:${MONGO_PASSWORD}@%container_namespace%-database:27017"
  JWT_SECRET: "${JWT_SECRET}"
  HELLO_MAIL_ADDRESS: "${HELLO_MAIL_ADDRESS}"
  HELLO_MAIL_PASSWORD: "${HELLO_MAIL_PASSWORD}"
  ADMIN_CREATION_PASSWORD: "${ADMIN_CREATION_PASSWORD}"
  AUDITORS_SERVICE_URL: "${AUDITORS_SERVICE_URL}"
  AUDITS_SERVICE_URL: "${AUDITS_SERVICE_URL}"
  CUSTOMERS_SERVICE_URL: "${CUSTOMERS_SERVICE_URL}"
  FILES_SERVICE_URL: "${FILES_SERVICE_URL}"
  MAIL_SERVICE_URL: "${MAIL_SERVICE_URL}"
  SEARCH_SERVICE_URL: "${SEARCH_SERVICE_URL}"
  USERS_SERVICE_URL: "${USERS_SERVICE_URL}"
  RENDERER_SERVICE_URL: "${RENDERER_SERVICE_URL}"
  NOTIFICATIONS_SERVICE_URL: "${NOTIFICATIONS_SERVICE_URL}"
  EVENTS_SERVICE_URL: "${EVENTS_SERVICE_URL}"
  FRONTEND: "${FRONTEND}"
  PROTOCOL: "${PROTOCOL}"
  FEEDBACK_EMAIL: "${FEEDBACK_EMAIL}"
  GITHUB_CLIENT_SECRET: "${GITHUB_CLIENT_SECRET}"
  GITHUB_CLIENT_ID: "${GITHUB_CLIENT_ID}"
  RUST_LOG: actix=info,reqwest=info,search=info,common=info,audits=trace
  TIMEOUT: "60"



services:
  %container_namespace%-binaries:
    build:
      context: ./
      dockerfile: ./Dockerfile
    volumes:
      - %volume_namespace%-binaries:/data/binaries

  %container_namespace%-users:
    depends_on:
      - %container_namespace%-binaries
    build: ./users
    %port_expose%:
      - 3001
    volumes:
      - %volume_namespace%-binaries:/data/binaries
    environment:
      VIRTUAL_HOST: dev.auditdb.io
      VIRTUAL_PATH: ~^/api/(user|auth|my_user|waiting_list)
      <<: *common-variables
    networks:
      - %network_namespace%-database
      - %proxy_network%

  %container_namespace%-customers:
    depends_on:
      - %container_namespace%-binaries
    build: ./customers
    %port_expose%:
      - 3002
    volumes:
      - %volume_namespace%-binaries:/data/binaries
    environment:
      VIRTUAL_HOST: dev.auditdb.io
      VIRTUAL_PATH: ~^/api/(customer|my_customer|project|my_project)
      <<: *common-variables
    networks:
      - %proxy_network%
      - %network_namespace%-database

  %container_namespace%-audits:
    depends_on:
      - %container_namespace%-binaries
    build: ./audits
    volumes:
      - %volume_namespace%-binaries:/data/binaries
    %port_expose%:
      - 3003
    environment:
      VIRTUAL_HOST: dev.auditdb.io
      VIRTUAL_PATH: ~^/api/(audit|my_audit|request|my_request|public_audits|no_customer_audit)
      <<: *common-variables
    networks:
      - %proxy_network%
      - %network_namespace%-database

  %container_namespace%-auditors:
    depends_on:
      - %container_namespace%-binaries
    build: ./auditors
    %port_expose%:
      - 3004
    volumes:
      - %volume_namespace%-binaries:/data/binaries
    environment:
      VIRTUAL_HOST: dev.auditdb.io
      VIRTUAL_PATH: ~^/api/(auditor|my_auditor|badge)
      <<: *common-variables
    networks:
      - %proxy_network%
      - %network_namespace%-database

  %container_namespace%-files:
    depends_on:
      - %container_namespace%-binaries
    build: ./files
    %port_expose%:
      - 3005
    volumes:
      - %volume_namespace%-binaries:/data/binaries
      - %volume_namespace%-files:/auditdb-files
    environment:
      VIRTUAL_HOST: dev.auditdb.io
      VIRTUAL_PATH: ~^/api/(file|notused1)
      <<: *common-variables
    networks:
      - %network_namespace%-database
      - %proxy_network%

  %container_namespace%-search:
    depends_on:
      - %container_namespace%-binaries
    build: ./search
    %port_expose%:
      - 3006
    volumes:
      - %volume_namespace%-binaries:/data/binaries
    environment:
      VIRTUAL_HOST: dev.auditdb.io
      VIRTUAL_PATH: ~^/api/(search|notused1)
      <<: *common-variables
    networks:
      - %proxy_network%
      - %network_namespace%-database

  %container_namespace%-mail:
    depends_on:
      - %container_namespace%-binaries
    build: ./mail
    %port_expose%:
      - 3007
    volumes:
      - %volume_namespace%-binaries:/data/binaries
    environment:
      VIRTUAL_HOST: dev.auditdb.io
      VIRTUAL_PATH: ~^/api/(mail|feedback|code)
      <<: *common-variables
    networks:
      - %proxy_network%
      - %network_namespace%-database

  %container_namespace%-notification:
    depends_on:
      - %container_namespace%-binaries
    build: ./notification
    %port_expose%:
      - 3008
    volumes:
      - %volume_namespace%-binaries:/data/binaries
    environment:
      VIRTUAL_HOST: dev.auditdb.io
      VIRTUAL_PATH: ~^/api/(send_notification|read_notification|unread_notifications)
      <<: *common-variables
    networks:
      - %proxy_network%
      - %network_namespace%-database

  %container_namespace%-telemetry:
    depends_on:
      - %container_namespace%-binaries
    build: ./telemetry
    %port_expose%:
      - 3009
    volumes:
      - %volume_namespace%-binaries:/data/binaries
    environment:
      VIRTUAL_HOST: dev.auditdb.io
      VIRTUAL_PATH: ~^/api/(telemetry|not_used1)
      <<: *common-variables
    networks:
      - %proxy_network%
      - %network_namespace%-database

  %container_namespace%-chat:
    depends_on:
      - %container_namespace%-binaries
    build: ./chat
    %port_expose%:
      - 3012
    volumes:
      - %volume_namespace%-binaries:/data/binaries
    environment:
      VIRTUAL_HOST: dev.auditdb.io
      VIRTUAL_PATH: ~^/api/(chat|not_used1)
      <<: *common-variables
    networks:
      - %proxy_network%
      - %network_namespace%-database

  %container_namespace%-event:
    depends_on:
      - %container_namespace%-binaries
    build: ./event
    %port_expose%:
      - 3010
    volumes:
      - %volume_namespace%-binaries:/data/binaries
    environment:
      VIRTUAL_HOST: dev.auditdb.io
      VIRTUAL_PATH: ~^/api/(notifications|event)
      <<: *common-variables
    networks:
      - %proxy_network%

  %container_namespace%-renderer:
    build:
      context: renderer
      dockerfile: Dockerfile
    %port_expose%:
      - 3015
    environment:
      VIRTUAL_HOST: dev.auditdb.io
      VIRTUAL_PATH: ~^/api/(generate-report|notused2)
    networks:
      - %proxy_network%
      - %network_namespace%-report

  %container_namespace%-report:
    build: ./report
    depends_on:
      - %container_namespace%-binaries
      - %container_namespace%-renderer
    %port_expose%:
      - 3011
    volumes:
      - %volume_namespace%-binaries:/data/binaries
    environment:
      VIRTUAL_HOST: dev.auditdb.io
      VIRTUAL_PATH: ~^/api/(report|notused2)
      <<: *common-variables
    networks:
      - %proxy_network%
      - %network_namespace%-report

  %container_namespace%-database:
    build: ./mongo
    environment:
      - MONGO_INITDB_ROOT_USERNAME=${MONGO_LOGIN}
      - MONGO_INITDB_ROOT_PASSWORD=${MONGO_PASSWORD}
    %open_database%:
      - 27017:27017
    volumes:
      - %volume_namespace%-database:/data/db
      - %volume_namespace%-database:/mongo_backup
    networks:
      - %network_namespace%-database
volumes:
  %volume_namespace%-database:
  %volume_namespace%-backup:
  %volume_namespace%-files:
  %volume_namespace%-binaries:
networks:
  %network_namespace%-report:
  %network_namespace%-database:
  %proxy_network%:"""


import subprocess

def clone_database_from(source, config, destination = None):
    destination = destination or "%volume_namespace%-database".replace("%volume_namespace%", config.volume_namespace)

    command = ["docker", "run", "--rm", "-it", "-v", f"{source}%:/from", "-v", f"{destination}:/to", "alpine", "ash", "-c", "cd /from ; to cp -av . /to"]
    print("running a command", command)
    subprocess.run(command)


# container_namespace, volume_namespace, network_namespace, load_database, open_database, with_proxy, export_database, import_database
def create_compose(config):
    global template
    template_instance = template
    port_expose = "ports"


    if config['open_database']:
        template_instance = template_instance.replace("%open_database%", "ports")
    else:
        template_instance = template_instance.replace("%open_database%", "expose")

    proxy_network_external = """
      external:
        true
    """
    proxy_newtwork_name = "%network_namespace%-nginx-proxy"

    if config['with_proxy']:
        port_expose = "expose"
        template_instance += proxy_network_external


    template_instance = template_instance.replace("%port_expose%", port_expose)

    template_instance = template_instance.replace("%proxy_network%", proxy_newtwork_name)

    template_instance = template_instance.replace("%container_namespace%", config['container_namespace'])
    template_instance = template_instance.replace("%volume_namespace%", config['volume_namespace'])
    template_instance = template_instance.replace("%network_namespace%", config['network_namespace'])
    return template_instance

preset = {
    "dev": {
        "load_database": False,
        "open_database": True,
        "with_proxy": False,
        "container_namespace": "dev",
        "volume_namespace": "dev",
        "network_namespace": "dev"
    },

    "prod": {
        "load_database": False,
        "open_database": False,
        "with_proxy": True,
        "container_namespace": "prod",
        "volume_namespace": "prod",
        "network_namespace": "prod"
    },
    "test": {
        "load_database": False,
        "open_database": True,
        "with_proxy": True,
        "container_namespace": "test",
        "volume_namespace": "test",
        "network_namespace": "test"
    },
    "preprod": {
        "load_database": True,
        "open_database": False,
        "with_proxy": True,
        "container_namespace": "preprod",
        "volume_namespace": "preprod",
        "network_namespace": "preprod"
    }
}


from dotenv import load_dotenv
import os


def get_config():
    global preset
    if os.environ["PRESET"] is not None:
        return preset[os.environ["PRESET"]]


    config = {
        "load_database": os.environ["LOAD_DATABASE"], 
        "open_dtabase": os.environ["OPEN_DATABASE"], 
        "with_proxy": os.environ["WITH_PROXY"], 
        "container_namespace": os.environ["CONTAINER_NAMESPACE"], 
        "volume_namespace": os.environ["VOLUME_NAMESPACE"], 
        "network_namespace": os.environ["NETWORK_NAMESPACE"]
    }
    return config
    
import sys

def main():
    load_dotenv()

    config = get_config()

    
    if len(sys.argv) > 1:
        clone_database_from(sys.argv[1], config)
        return

    compose = create_compose(config)

    
    with open("docker-compose.yml", "w") as f:
        f.write(compose)

if __name__ == "__main__":
    main()
