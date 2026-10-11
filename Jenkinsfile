// Runs in the home lab's Jenkins. Each stage gets its own pod, pinned to the
// node that holds its build cache under /home/k8s/metorex-ci-cache, so the
// cache is warm on every run.

def buildPod(String node, String cpuRequest, String cpuLimit) {
  return """
    apiVersion: v1
    kind: Pod
    spec:
      nodeSelector:
        kubernetes.io/hostname: ${node}
      imagePullSecrets:
        - name: harbor-metorex
      securityContext:
        runAsUser: 1000
        runAsGroup: 1000
      containers:
        - name: ci
          image: harbor.home.gregdonald.com/metorex/ci:latest
          imagePullPolicy: Always
          command: ["sleep"]
          args: ["infinity"]
          env:
            - name: CARGO_HOME
              value: /cache/cargo
          resources:
            requests:
              cpu: "${cpuRequest}"
              memory: 8Gi
            limits:
              cpu: "${cpuLimit}"
              memory: 8Gi
          volumeMounts:
            - name: cache
              mountPath: /cache
      volumes:
        - name: cache
          hostPath:
            path: /home/k8s/metorex-ci-cache
            type: Directory
  """
}

pipeline {
  agent none

  stages {
    stage('checks') {
      parallel {
        stage('test') {
          agent {
            kubernetes {
              defaultContainer 'ci'
              yaml buildPod('risa', '3', '6')
            }
          }
          environment {
            CARGO_TARGET_DIR = '/cache/target-test'
          }
          steps {
            sh 'cargo fmt -- --check'
            sh 'cargo clippy --all-targets -- -D warnings'
            sh 'scripts/misplaced_tests.sh'
            sh 'cargo build'
            sh 'cargo test'
          }
        }

        stage('ruby-spec') {
          agent {
            kubernetes {
              defaultContainer 'ci'
              yaml buildPod('risa', '4', '8')
            }
          }
          environment {
            CARGO_TARGET_DIR = '/cache/target-spec'
            JOBS = '10'
          }
          steps {
            sh 'scripts/run_ruby_spec.sh'
          }
        }

        stage('coverage') {
          agent {
            kubernetes {
              defaultContainer 'ci'
              yaml buildPod('jaros', '8', '8')
            }
          }
          environment {
            CARGO_TARGET_DIR = '/cache/target-cov'
          }
          steps {
            sh "cargo llvm-cov --all-features --ignore-filename-regex 'ruby/' --cobertura --output-path cobertura.xml"
          }
        }
      }
    }
  }
}
