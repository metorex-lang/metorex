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

// BuildKit's rootless example for Kubernetes, with the Harbor robot's
// credentials mounted as a Docker config so the build can push.
def imageBuildPod = '''
  apiVersion: v1
  kind: Pod
  spec:
    securityContext:
      runAsUser: 1000
      runAsGroup: 1000
    containers:
      - name: buildkit
        image: moby/buildkit:v0.33.1-rootless
        command: ["sleep"]
        args: ["infinity"]
        env:
          - name: BUILDKITD_FLAGS
            value: --oci-worker-no-process-sandbox
          - name: DOCKER_CONFIG
            value: /home/user/.docker-harbor
        securityContext:
          seccompProfile:
            type: Unconfined
          appArmorProfile:
            type: Unconfined
        volumeMounts:
          - name: buildkitd
            mountPath: /home/user/.local/share/buildkit
          - name: harbor
            mountPath: /home/user/.docker-harbor
            readOnly: true
    volumes:
      - name: buildkitd
        emptyDir: {}
      - name: harbor
        secret:
          secretName: harbor-metorex
          items:
            - key: .dockerconfigjson
              path: config.json
'''

pipeline {
  agent none

  options {
    disableConcurrentBuilds(abortPrevious: true)
  }

  parameters {
    booleanParam(name: 'BUILD_CI_IMAGE', defaultValue: false, description: 'Rebuild and push harbor.home.gregdonald.com/metorex/ci:latest before the checks')
  }

  stages {
    stage('ci-image') {
      when {
        beforeAgent true
        anyOf {
          expression { params.BUILD_CI_IMAGE }
          allOf {
            branch 'main'
            changeset 'docker/Dockerfile'
          }
        }
      }
      agent {
        kubernetes {
          defaultContainer 'buildkit'
          yaml imageBuildPod
        }
      }
      steps {
        sh '''
          buildctl-daemonless.sh build \
            --frontend dockerfile.v0 \
            --local context=docker \
            --local dockerfile=docker \
            --output type=image,name=harbor.home.gregdonald.com/metorex/ci:latest,push=true
        '''
      }
    }

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
            // run_ruby_spec.sh builds only when no binary exists, and the
            // cached one may be from older sources or another checkout path.
            sh 'cargo build'
            sh 'scripts/run_ruby_spec.sh'
          }
        }

        // The same tests with METOREX_ENGINE=bytecode. Each metorex process
        // writes its engine report into engine-report/, and the total is
        // kept with the build.
        stage('test-bytecode') {
          agent {
            kubernetes {
              defaultContainer 'ci'
              yaml buildPod('risa', '3', '6')
            }
          }
          environment {
            CARGO_TARGET_DIR = '/cache/target-test-bytecode'
            METOREX_ENGINE = 'bytecode'
          }
          steps {
            sh 'mkdir -p engine-report'
            sh 'cargo build'
            sh 'METOREX_ENGINE_REPORT="$PWD/engine-report" cargo test'
          }
          post {
            always {
              sh 'scripts/engine_report_summary.sh engine-report > engine-report-tests.txt || true'
              archiveArtifacts artifacts: 'engine-report-tests.txt', allowEmptyArchive: true
            }
          }
        }

        stage('ruby-spec-bytecode') {
          agent {
            kubernetes {
              defaultContainer 'ci'
              yaml buildPod('risa', '4', '8')
            }
          }
          environment {
            CARGO_TARGET_DIR = '/cache/target-spec-bytecode'
            JOBS = '10'
            METOREX_ENGINE = 'bytecode'
          }
          steps {
            sh 'mkdir -p engine-report'
            sh 'cargo build'
            sh 'METOREX_ENGINE_REPORT="$PWD/engine-report" scripts/run_ruby_spec.sh'
          }
          post {
            always {
              sh 'scripts/engine_report_summary.sh engine-report > engine-report-ruby-spec.txt || true'
              archiveArtifacts artifacts: 'engine-report-ruby-spec.txt', allowEmptyArchive: true
            }
          }
        }

        stage('rspec-suites') {
          agent {
            kubernetes {
              defaultContainer 'ci'
              yaml buildPod('risa', '4', '8')
            }
          }
          environment {
            CARGO_TARGET_DIR = '/cache/target-rspec'
            JOBS = '10'
          }
          steps {
            sh 'cargo build'
            sh 'scripts/compare_fast_paths.sh'
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
