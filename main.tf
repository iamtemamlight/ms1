# Proprietary Infrastructure for Allbright DeFi Software Engineering PLC
# Module for Automated Runner Fleet Expansion

variable "region_name" {
  type        = string
  description = "The target deployment region (e.g., 'us-east-eth-1')"
}

variable "runner_count" {
  type    = number
  default = 50
}

variable "central_c2_addr" {
  type        = string
  description = "The gRPC address of the central Command & Control server for Allbright"
}

resource "kubernetes_namespace" "fleet_namespace" {
  metadata {
    name = "allbright-fleet-${var.region_name}"
    labels = {
      owner = "allbright-defi"
      tier  = "production"
    }
  }
}

resource "kubernetes_deployment" "regional_aggregator" {
  metadata {
    name      = "regional-aggregator"
    namespace = kubernetes_namespace.fleet_namespace.metadata[0].name
  }

  spec {
    replicas = 1
    selector {
      match_labels = {
        app = "regional-aggregator"
      }
    }
    template {
      metadata {
        labels = {
          app = "regional-aggregator"
        }
      }
      spec {
        container {
          image = "allbright-registry.internal/regional-aggregator:latest"
          name  = "aggregator"
          env {
            name  = "REGION_ID"
            value = var.region_name
          }
          env {
            name  = "CENTRAL_C2_ADDR"
            value = var.central_c2_addr
          }
          resources {
            limits = {
              cpu    = "2"
              memory = "4Gi"
            }
          }
          port {
            container_port = 50052
          }
        }
      }
    }
  }
}

resource "kubernetes_deployment" "runner_node" {
  count = var.runner_count
  metadata {
    name      = "sovereign-runner-${count.index}"
    namespace = kubernetes_namespace.fleet_namespace.metadata[0].name
  }

  spec {
    replicas = 1
    selector {
      match_labels = {
        app = "sovereign-runner"
      }
    }
    template {
      metadata {
        labels = {
          app = "sovereign-runner"
        }
      }
      spec {
        container {
          image = "allbright-registry.internal/sovereign-hybrid:latest"
          name  = "engine"
          # Hardware bypass and AVX-512 optimization flags
          resources {
            limits = {
              cpu    = "4"
              memory = "8Gi"
            }
          }
          env {
            name  = "RUNNER_ID"
            value = "node-${var.region_name}-${count.index}"
          }
        }

        # Proprietary Telemetry Side-car for automated monitoring
        container {
          image = "allbright-registry.internal/telemetry-sidecar:latest"
          name  = "telemetry"
          env {
            name  = "RUNNER_ID"
            value = "node-${var.region_name}-${count.index}"
          }
          env {
            name  = "C2_SERVER_ADDR"
            value = "http://regional-aggregator-svc.${kubernetes_namespace.fleet_namespace.metadata[0].name}.svc.cluster.local:50052"
          }
        }
      }
    }
  }
}

resource "kubernetes_service" "aggregator_service" {
  metadata {
    name      = "regional-aggregator-svc"
    namespace = kubernetes_namespace.fleet_namespace.metadata[0].name
  }
  spec {
    selector = {
      app = "regional-aggregator"
    }
    port {
      port        = 50052
      target_port = 50052
      protocol    = "TCP"
    }
  }
}