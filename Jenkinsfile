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
            - name: LANG
              value: C.UTF-8
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
      options {
        timeout(time: 30, unit: 'MINUTES')
      }
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

    // The stages run one after another, so each one's output stands alone
    // in the console. A failing stage marks the build failed and the stages
    // after it still run, so one run reports every failure. Each stage's
    // test reports appear under Tests, with the output of each failure.
    stage('lint') {
      options {
        timeout(time: 10, unit: 'MINUTES')
      }
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
        catchError(buildResult: 'FAILURE', stageResult: 'FAILURE') {
          sh 'cargo fmt -- --check'
          sh 'cargo clippy --all-targets -- -D warnings'
          sh 'scripts/misplaced_tests.sh'
        }
      }
    }

    stage('test') {
      options {
        timeout(time: 30, unit: 'MINUTES')
      }
      agent {
        kubernetes {
          defaultContainer 'ci'
          yaml buildPod('risa', '3', '6')
        }
      }
      environment {
        CARGO_TARGET_DIR = '/cache/target-test'
        RUST_BACKTRACE = '1'
      }
      // Each metorex process writes its engine report into
      // engine-report/, and the total is kept with the build.
      steps {
        catchError(buildResult: 'FAILURE', stageResult: 'FAILURE') {
          sh 'mkdir -p engine-report test-results'
          sh 'cargo build'
          sh 'METOREX_ENGINE_REPORT="$PWD/engine-report" cargo nextest run --profile ci'
          sh 'cargo test --doc'
        }
      }
      post {
        always {
          sh 'cp target/nextest/ci/junit.xml test-results/cargo-test.xml || true'
          junit testResults: 'test-results/*.xml', allowEmptyResults: true
          sh 'scripts/engine_report_summary.sh engine-report > engine-report-tests.txt || true'
          archiveArtifacts artifacts: 'engine-report-tests.txt', allowEmptyArchive: true
        }
      }
    }

    stage('ruby-spec') {
      options {
        timeout(time: 40, unit: 'MINUTES')
      }
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
        catchError(buildResult: 'FAILURE', stageResult: 'FAILURE') {
          // run_ruby_spec.sh builds only when no binary exists, and the
          // cached one may be from older sources or another checkout path.
          sh 'mkdir -p engine-report test-results'
          sh 'cargo build'
          sh 'METOREX_ENGINE_REPORT="$PWD/engine-report" SPEC_JUNIT="$PWD/test-results/ruby-spec.xml" scripts/run_ruby_spec.sh'
        }
      }
      post {
        always {
          junit testResults: 'test-results/*.xml', allowEmptyResults: true
          sh 'scripts/engine_report_summary.sh engine-report > engine-report-ruby-spec.txt || true'
          archiveArtifacts artifacts: 'engine-report-ruby-spec.txt', allowEmptyArchive: true
        }
      }
    }

    // The same suites with the JIT compiling every unit on its first
    // run, so each instruction runs from machine code.
    stage('jit') {
      options {
        timeout(time: 60, unit: 'MINUTES')
      }
      agent {
        kubernetes {
          defaultContainer 'ci'
          yaml buildPod('risa', '4', '8')
        }
      }
      environment {
        CARGO_TARGET_DIR = '/cache/target-jit'
        JOBS = '10'
        METOREX_JIT = '1'
        METOREX_JIT_CALL_THRESHOLD = '1'
        RUST_BACKTRACE = '1'
      }
      steps {
        catchError(buildResult: 'FAILURE', stageResult: 'FAILURE') {
          sh 'mkdir -p test-results'
          sh 'cargo build'
          sh 'cargo nextest run --profile ci'
          sh 'SPEC_JUNIT="$PWD/test-results/jit-ruby-spec.xml" scripts/run_ruby_spec.sh'
        }
      }
      post {
        always {
          sh 'cp target/nextest/ci/junit.xml test-results/jit-cargo-test.xml || true'
          junit testResults: 'test-results/*.xml', allowEmptyResults: true
        }
      }
    }

    stage('rspec-suites') {
      options {
        timeout(time: 120, unit: 'MINUTES')
      }
      agent {
        kubernetes {
          defaultContainer 'ci'
          yaml buildPod('risa', '4', '8')
        }
      }
      // The suites run on the `suites` build, which runs them about ten
      // times as fast as a debug build does.
      environment {
        CARGO_TARGET_DIR = '/cache/target-rspec'
        METOREX = '/cache/target-rspec/suites/metorex'
        JOBS = '8'
      }
      steps {
        catchError(buildResult: 'FAILURE', stageResult: 'FAILURE') {
          sh 'cargo build --profile suites'
          sh 'scripts/install_rspec_suite_gems.sh'
          sh 'scripts/compare_fast_paths.sh'
        }
      }
    }

    // Each group of gem suites is a branch of its own, so a failing group
    // shows by name. A branch fails when a file passes fewer examples than
    // scripts/gem_suite_results.tsv records for Metorex.
    stage('gems') {
      options {
        timeout(time: 30, unit: 'MINUTES')
      }
      agent {
        kubernetes {
          defaultContainer 'ci'
          yaml buildPod('risa', '4', '8')
        }
      }
      environment {
        CARGO_TARGET_DIR = '/cache/target-rspec'
        METOREX = '/cache/target-rspec/suites/metorex'
        JOBS = '8'
        CHECK = '1'
      }
      steps {
        catchError(buildResult: 'FAILURE', stageResult: 'FAILURE') {
          // power_assert's gemspec lists its files through git.
          sh 'git --version'
          sh 'cargo build --profile suites'
          script {
            parallel(
              'bundled gems': {
                sh 'scripts/run_gem_suites.sh minitest test-unit power_assert rake rexml rss net-imap net-pop net-smtp'
              }
            )
          }
        }
      }
    }

    stage('coverage') {
      options {
        timeout(time: 20, unit: 'MINUTES')
      }
      agent {
        kubernetes {
          defaultContainer 'ci'
          yaml buildPod('jaros', '8', '8')
        }
      }
      environment {
        CARGO_TARGET_DIR = '/cache/target-cov'
        RUST_BACKTRACE = '1'
      }
      steps {
        catchError(buildResult: 'FAILURE', stageResult: 'FAILURE') {
          sh 'mkdir -p test-results'
          sh "cargo llvm-cov nextest --profile ci --all-features --ignore-filename-regex 'ruby/' --cobertura --output-path cobertura.xml"
        }
      }
      post {
        always {
          sh 'cp target/nextest/ci/junit.xml test-results/coverage-cargo-test.xml || true'
          junit testResults: 'test-results/*.xml', allowEmptyResults: true
        }
      }
    }
  }
}
