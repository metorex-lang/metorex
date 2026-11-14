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
          image: harbor.home.gregdonald.com/metorex/ci:${env.BUILD_TAG}
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

  stages {
    stage('ci-image') {
      options {
        timeout(time: 30, unit: 'MINUTES')
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
            --import-cache type=registry,ref=harbor.home.gregdonald.com/metorex/ci:buildcache \
            --export-cache type=registry,ref=harbor.home.gregdonald.com/metorex/ci:buildcache,mode=max \
            --output "type=image,\\"name=harbor.home.gregdonald.com/metorex/ci:$BUILD_TAG,harbor.home.gregdonald.com/metorex/ci:latest\\",push=true"
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
      // times as fast as a debug build does. A spec file whose counts
      // differ from MRI's in scripts/rspec_suite_results.tsv fails the stage.
      environment {
        CARGO_TARGET_DIR = '/cache/target-rspec'
        METOREX = '/cache/target-rspec/suites/metorex'
        JOBS = '8'
        CHECK = '1'
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
    // scripts/gem_suite_results.tsv records for Metorex, and prints the row
    // to write for a file that passes more. The branches share the pod's
    // CPUs, so each runs as many files at once as its share of the work.
    stage('gems') {
      options {
        timeout(time: 180, unit: 'MINUTES')
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
        CHECK = '1'
        // rexml's linear performance tests parse documents of up to 200,000
        // characters, which takes this node longer than the default 300.
        FILE_TIMEOUT = '900'
      }
      steps {
        catchError(buildResult: 'FAILURE', stageResult: 'FAILURE') {
          // power_assert's gemspec lists its files through git.
          sh 'git --version'
          sh 'cargo build --profile suites'
          sh 'scripts/install_gem_suite_gems.sh'
          script {
            parallel(
              'bundled gems': {
                sh 'JOBS=1 scripts/run_gem_suites.sh minitest test-unit power_assert rake rexml rss net-imap net-pop net-smtp'
              },
              'gems Ruby ships': {
                sh 'JOBS=1 scripts/run_gem_suites.sh racc base64 json bigdecimal'
              },
              'rack and HTTP': {
                sh 'JOBS=1 scripts/run_gem_suites.sh rack rack-test sinatra/rack-protection faraday faraday-net_http multipart-post excon httpclient http-cookie domain_name connection_pool websocket-extensions'
              },
              'text, data, and time': {
                sh 'JOBS=1 scripts/run_gem_suites.sh i18n tzinfo multi_json multi_xml builder mime-types mini_mime mail addressable public_suffix jwt dotenv netrc rubyzip'
              },
              'command line and developer tools': {
                sh 'JOBS=1 scripts/run_gem_suites.sh thor rainbow ruby-progressbar unicode-display_width parallel method_source coderay pry simplecov docile mini_portile2'
              },
              'ruby parsing and linting': {
                sh 'JOBS=3 scripts/run_gem_suites.sh ast parser regexp_parser rubocop-ast rubocop'
              },
              'templates, loading, and concurrency': {
                sh 'JOBS=1 scripts/run_gem_suites.sh tilt erubi zeitwerk sprockets concurrent-ruby thread_safe'
              },
              'cloud clients': {
                sh 'JOBS=1 scripts/run_gem_suites.sh aws-sdk-ruby/gems/aws-sdk-core aws-sdk-ruby/gems/aws-sigv4 aws-sdk-ruby/gems/aws-partitions aws-sdk-ruby/gems/aws-eventstream aws-sdk-ruby/gems/aws-sdk-s3 aws-sdk-ruby/gems/aws-sdk-kms jmespath googleauth signet'
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
