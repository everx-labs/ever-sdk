G_giturl = "git@github.com:tonlabs/TON-SDK.git"
G_gitcred = 'TonJenSSH'
G_docker_creds = 'dockerhubLanin'
C_PROJECT = "NotSet"
C_COMMITER = "NotSet"
C_HASH = "NotSet"
C_TEXT = "NotSet"
G_binversion = "NotSet"
G_tsnj_build = true
G_tsnj_deploy = true
G_images = [:]
G_branches = [:]
G_params = null
DiscordURL = "https://discordapp.com/api/webhooks/496992026932543489/4exQIw18D4U_4T0H76bS3Voui4SyD7yCQzLP9IRQHKpwGRJK1-IFnyZLyYzDmcBKFTJw"

def getVar(Gvar) {
    return Gvar
}

def isUpstream() {
    return currentBuild.getBuildCauses()[0]._class.toString() == 'hudson.model.Cause$UpstreamCause'
}

def buildImagesMap() {
    if (params.image_ton_types == '') {
        G_images.put('ton-types', "tonlabs/ton-types:ton-sdk-${GIT_COMMIT}")
    } else {
        G_images.put('ton-types', params.image_ton_types)
    }

    if (params.image_ton_labs_types == '') {
        G_images.put('ton-labs-types', "tonlabs/ton-labs-types:ton-sdk-${GIT_COMMIT}")
    } else {
        G_images.put('ton-labs-types', params.image_ton_labs_types)
    }

    if (params.image_ton_block == '') {
        G_images.put('ton-block', "tonlabs/ton-block:ton-sdk-${GIT_COMMIT}")
    } else {
        G_images.put('ton-block', params.image_ton_block)
    }

    if (params.image_ton_labs_block == '') {
        G_images.put('ton-labs-block', "tonlabs/ton-labs-block:ton-sdk-${GIT_COMMIT}")
    } else {
        G_images.put('ton-labs-block', params.image_ton_labs_block)
    }

    if (params.image_ton_vm == '') {
        G_images.put('ton-vm', "tonlabs/ton-vm:ton-sdk-${GIT_COMMIT}")
    } else {
        G_images.put('ton-vm', params.image_ton_vm)
    }

    if (params.image_ton_labs_vm == '') {
        G_images.put('ton-labs-vm', "tonlabs/ton-labs-vm:ton-sdk-${GIT_COMMIT}")
    } else {
        G_images.put('ton-labs-vm', params.image_ton_labs_vm)
    }

    if (params.image_ton_labs_abi == '') {
        G_images.put('ton-labs-abi', "tonlabs/ton-labs-abi:ton-sdk-${GIT_COMMIT}")
    } else {
        G_images.put('ton-labs-abi', params.image_ton_labs_abi)
    }

    if (params.image_ton_executor == '') {
        G_images.put('ton-executor', "tonlabs/ton-executor:ton-sdk-${GIT_COMMIT}")
    } else {
        G_images.put('ton-executor', params.image_ton_executor)
    }

    if (params.image_ton_sdk == '') {
        G_images.put('ton-sdk', "tonlabs/ton-sdk:${GIT_COMMIT}")
    } else {
        G_images.put('ton-sdk', params.image_ton_sdk)
    }

    if (params.image_tvm_linker == '') {
        G_images.put('tvm-linker', "tonlabs/tvm_linker:ton-sdk-${GIT_COMMIT}")
    } else {
        G_images.put('tvm-linker', params.image_tvm_linker)
    }
}

def buildBranchesMap() {
    if (params.branch_ton_types == '') {
        G_branches.put('ton-types', "master")
    } else {
        G_branches.put('ton-types', params.branch_ton_types)
    }
    
    if (params.branch_ton_labs_types == '') {
        G_branches.put('ton-labs-types', "release-candidate")
    } else {
        G_branches.put('ton-labs-types', params.branch_ton_labs_types)
    }

    if (params.branch_ton_block == '') {
        G_branches.put('ton-block', "master")
    } else {
        G_branches.put('ton-block', params.branch_ton_block)
    }

    if (params.branch_ton_labs_block == '') {
        G_branches.put('ton-labs-block', "release-candidate")
    } else {
        G_branches.put('ton-labs-block', params.branch_ton_labs_block)
    }

    if (params.branch_ton_vm == '') {
        G_branches.put('ton-vm', "master")
    } else {
        G_branches.put('ton-vm', params.branch_ton_vm)
    }

    if (params.branch_ton_labs_vm == '') {
        G_branches.put('ton-labs-vm', "release-candidate")
    } else {
        G_branches.put('ton-labs-vm', params.branch_ton_labs_vm)
    }

    if (params.branch_ton_labs_abi == '') {
        G_branches.put('ton-labs-abi', "master")
    } else {
        G_branches.put('ton-labs-abi', params.branch_ton_labs_abi)
    }

    if (params.branch_ton_executor == '') {
        G_branches.put('ton-executor', "master")
    } else {
        G_branches.put('ton-executor', params.branch_ton_executor)
    }

    if (params.branch_ton_sdk == '') {
        G_branches.put('ton-sdk', "${env.BRANCH_NAME}")
    } else {
        G_branches.put('ton-sdk', params.branch_ton_sdk)
    }

    if (params.branch_tvm_linker == '') {
        G_branches.put('tvm-linker', "master")
    } else {
        G_branches.put('tvm-linker', params.branch_tvm_linker)
    }

    if (params.branch_sol2tvm == '') {
        G_branches.put('sol2tvm', "master")
    } else {
        G_branches.put('sol2tvm', params.branch_sol2tvm)
    }
}

def buildParams() {
    buildImagesMap()
    if(!isUpstream() && GIT_BRANCH != 'master' && !(GIT_BRANCH ==~ '^PR-[0-9]+')) {
        G_images['ton-types'] = 'tonlabs/ton-types:latest'
        G_images['ton-labs-types'] = 'tonlabs/ton-labs-types:latest'
        G_images['ton-block'] = 'tonlabs/ton-block:latest'
        G_images['ton-labs-block'] = 'tonlabs/ton-labs-block:latest'
        G_images['ton-vm'] = 'tonlabs/ton-vm:latest'
        G_images['ton-labs-vm'] = 'tonlabs/ton-labs-vm:latest'
        G_images['ton-executor'] = 'tonlabs/ton-executor:latest'
        G_images['ton-labs-abi'] = 'tonlabs/ton-labs-abi:latest'
    }
    buildBranchesMap()
    G_params = []
    params.each { key, value ->
        def item = null
        def nKey = key.toLowerCase().replaceAll('branch_', '').replaceAll('image_', '').replaceAll('_','-')
        if(key ==~ '^branch_(.)+') {
            item = string("name": key, "value": G_branches["${nKey}"])
        } else {
            if(key ==~ '^image_(.)+') {
                item = string("name": key, "value": G_images["${nKey}"])
            } else {
                if(key == 'common_version') {
                    item = string("name": 'key', "value": G_binversion)
                } else {
                    item = string("name": key, "value": value)
                }
            }
        }
        G_params.push(item)
    }
}

pipeline {
    agent {
        node 'master'
    }
    tools {nodejs "Node12.8.0"}
    options {
        buildDiscarder logRotator(artifactDaysToKeepStr: '', artifactNumToKeepStr: '', daysToKeepStr: '', numToKeepStr: '20')
        disableConcurrentBuilds()
        parallelsAlwaysFailFast()
    }
    parameters {
        string(
            name:'common_version',
            defaultValue: '',
            description: 'Common version'
        )
        string(
            name:'branch_ton_types',
            defaultValue: 'master',
            description: 'ton-types branch for dependency test'
        )
        string(
            name:'image_ton_types',
            defaultValue: '',
            description: 'ton-types image name'
        )
        string(
            name:'branch_ton_labs_types',
            defaultValue: '',
            description: 'ton-labs-types branch for dependency test'
        )
        string(
            name:'image_ton_labs_types',
            defaultValue: '',
            description: 'ton-labs-types image name'
        )
        string(
            name:'branch_ton_block',
            defaultValue: 'master',
            description: 'ton-block branch'
        )
        string(
            name:'image_ton_block',
            defaultValue: '',
            description: 'ton-block image name'
        )
        string(
            name:'branch_ton_labs_block',
            defaultValue: '',
            description: 'ton-labs-block branch'
        )
        string(
            name:'image_ton_labs_block',
            defaultValue: '',
            description: 'ton-labs-block image name'
        )
        string(
            name:'branch_ton_vm',
            defaultValue: 'master',
            description: 'ton-vm branch'
        )
        string(
            name:'image_ton_vm',
            defaultValue: '',
            description: 'ton-vm image name'
        )
        string(
            name:'branch_ton_labs_vm',
            defaultValue: '',
            description: 'ton-labs-vm branch'
        )
        string(
            name:'image_ton_labs_vm',
            defaultValue: '',
            description: 'ton-labs-vm image name'
        )
        string(
            name:'branch_ton_labs_abi',
            defaultValue: 'master',
            description: 'ton-labs-abi branch'
        )
        string(
            name:'image_ton_labs_abi',
            defaultValue: '',
            description: 'ton-labs-abi image name'
        )
        string(
            name:'branch_ton_executor',
            defaultValue: 'master',
            description: 'ton-executor branch'
        )
        string(
            name:'image_ton_executor',
            defaultValue: '',
            description: 'ton-executor image name'
        )
        string(
            name:'branch_tvm_linker',
            defaultValue: 'master',
            description: 'tvm-linker branch'
        )
        string(
            name:'image_tvm_linker',
            defaultValue: '',
            description: 'tvm-linker image name'
        )
        string(
            name:'branch_ton_sdk',
            defaultValue: 'master',
            description: 'ton-sdk branch'
        )
        string(
            name:'image_ton_sdk',
            defaultValue: '',
            description: 'ton-sdk image name'
        )
        string(
            name:'branch_sol2tvm',
            defaultValue: 'master',
            description: 'sol2tvm branch'
        )
    }
    stages {
        stage('Versioning') {
            steps {
                script {
                    withAWS(credentials: 'CI_bucket_writer', region: 'eu-central-1') {
                        identity = awsIdentity()
                        s3Download bucket: 'sdkbinaries.tonlabs.io', file: 'version.json', force: true, path: 'version.json'
                    }
                    def folders = """ton_sdk \
ton_client/client \
ton_client/platforms/ton-client-node-js \
ton_client/platforms/ton-client-react-native \
ton_client/platforms/ton-client-web"""
                    if(params.common_version) {
                        G_binversion = sh (script: "node tonVersion.js --set ${params.common_version} ${folders}", returnStdout: true).trim()
                    } else {
                        G_binversion = sh (script: "node tonVersion.js ${folders}", returnStdout: true).trim()
                    }

                    if(!isUpstream() && (GIT_BRANCH == 'master' || GIT_BRANCH ==~ '^PR-[0-9]+' || GIT_BRANCH == "${getVar(G_binversion)}-rc")) {
                        withAWS(credentials: 'CI_bucket_writer', region: 'eu-central-1') {
                            identity = awsIdentity()
                            s3Upload \
                                bucket: 'sdkbinaries.tonlabs.io', \
                                includePathPattern:'version.json', path: '', \
                                workingDir:'.'
                        }
                    }
                }
            }
        }
        stage('Initialize') {
            steps {
                script {
                    G_gitproject = G_giturl.substring(15,G_giturl.length()-4)
                    G_gitproject_dir = G_gitproject.substring(8, G_gitproject.length())
                    C_TEXT = sh (script: "git show -s --format=%s ${GIT_COMMIT}", \
                        returnStdout: true).trim()
                    C_AUTHOR = sh (script: "git show -s --format=%an ${GIT_COMMIT}", \
                        returnStdout: true).trim()
                    C_COMMITER = sh (script: "git show -s --format=%cn ${GIT_COMMIT}", \
                        returnStdout: true).trim()
                    C_HASH = sh (script: "git show -s --format=%h ${GIT_COMMIT}", \
                        returnStdout: true).trim()
                    C_PROJECT = G_giturl.substring(15,G_giturl.length()-4)
                    C_GITURL = sh (script: "echo ${GIT_URL}",returnStdout: true).trim()
                    C_GITCOMMIT = sh (script: "echo ${GIT_COMMIT}", \
                        returnStdout: true).trim()
                }
                echo "Version: ${getVar(G_binversion)}."
                echo "Branch: ${GIT_BRANCH}"
                echo "Possible RC: ${getVar(G_binversion)}-rc"

                buildParams()
                echo "${G_params}"
            }
        }
        stage('Before stages') {
            when {
                expression {
                    return !isUpstream() && (GIT_BRANCH == 'master' || GIT_BRANCH ==~ '^PR-[0-9]+' || GIT_BRANCH == "${getVar(G_binversion)}-rc")
                }
            }
            steps {
                script {
                    def beforeParams = G_params
                    beforeParams.push(string("name": "project_name", "value": "ton-sdk"))
                    beforeParams.push(string("name": "stage", "value": "before"))
                    build job: 'Builder/build-flow', parameters: beforeParams
                }
            }
        }
        stage('Switch to file source') {
            steps {
                sh """
                    node pathFix.js ton_sdk/Cargo.toml \"ton_abi = {.*\" \"ton_abi = { path = \\\"/tonlabs/ton-labs-abi\\\" }\"
                    node pathFix.js ton_sdk/Cargo.toml \"ton_block = {.*\" \"ton_block = { path = \\\"/tonlabs/ton-labs-block\\\" }\"
                    node pathFix.js ton_sdk/Cargo.toml \"ton_vm = {.*\" \"ton_vm = { path = \\\"/tonlabs/ton-labs-vm\\\", default-features = false }\"
                    node pathFix.js ton_sdk/Cargo.toml \"ton_types = {.*\" \"ton_types = { path = \\\"/tonlabs/ton-labs-types\\\" }\"
                    node pathFix.js ton_sdk/Cargo.toml \"ton_executor = {.*\" \"ton_executor = { path = \\\"/tonlabs/ton-executor\\\" }\"
                    node pathFix.js ton_sdk/Cargo.toml \"git = \\\"ssh://git@github.com/tonlabs/ton-executor.git\\\"\" \"path = \\\"/tonlabs/ton-executor\\\"\"
                    node pathFix.js ton_client/client/Cargo.toml \"ton_block = {.*\" \"ton_block = { path = \\\"/tonlabs/ton-labs-block\\\" }\"
                    node pathFix.js ton_client/client/Cargo.toml \"ton_vm = {.*\" \"ton_vm = { path = \\\"/tonlabs/ton-labs-vm\\\", default-features = false }\"
                    node pathFix.js ton_client/client/Cargo.toml \"ton_types = {.*\" \"ton_types = { path = \\\"/tonlabs/ton-labs-types\\\" }\"
                """
            }
        }
        stage('Build sources image') {
            steps {
                script {
                    docker.withRegistry('', G_docker_creds) {
                        sshagent (credentials: [G_gitcred]) {
                            withEnv(["DOCKER_BUILDKIT=1", "BUILD_INFO=${env.BUILD_TAG}:${GIT_COMMIT}"]) {
                                src_image = docker.build(
                                    "${G_images['ton-sdk']}",
                                    "--pull --label \"git-commit=\${GIT_COMMIT}\" --target ton-sdk-src ."
                                )
                            }
                        }
                        docker.image("${G_images['ton-sdk']}").push()
                    }
                }
            }
        }
        stage('Build common sources for agents') {
            agent {
                dockerfile {
                    registryCredentialsId "${G_docker_creds}"
                    additionalBuildArgs "--pull --target ton-sdk-full " + 
                                        "--build-arg \"TON_LABS_TYPES_IMAGE=${G_images['ton-labs-types']}\" " +
                                        "--build-arg \"TON_LABS_BLOCK_IMAGE=${G_images['ton-labs-block']}\" " + 
                                        "--build-arg \"TON_LABS_VM_IMAGE=${G_images['ton-labs-vm']}\" " + 
                                        "--build-arg \"TON_LABS_ABI_IMAGE=${G_images['ton-labs-abi']}\" " + 
                                        "--build-arg \"TON_EXECUTOR_IMAGE=${G_images['ton-executor']}\" " +
                                        "--build-arg \"TON_SDK_IMAGE=${G_images['ton-sdk']}\""
                }
            }
            steps {
                script {
                    sh """
                        zip -9 -r ton-sdk-src.zip /tonlabs/*
                        ls -la ton-sdk-src.zip
                        chown jenkins:jenkins ton-sdk-src.zip
                    """
                    stash includes: '**/ton-sdk-src.zip', name: 'ton-sdk-src'
                }
            }
        }
        stage('Building...') {
            failFast true
            parallel {
                stage('Parallel stages') {
                    when {
                        expression {
                            return !isUpstream() && (GIT_BRANCH == 'master' || GIT_BRANCH ==~ '^PR-[0-9]+' || GIT_BRANCH == "${getVar(G_binversion)}-rc")
                        }
                    }
                    steps {
                        script {
                            def intimeParams = G_params
                            intimeParams.push(string("name": "project_name", "value": "ton-sdk"))
                            intimeParams.push(string("name": "stage", "value": "in_time"))
                            build job: 'Builder/build-flow', parameters: intimeParams
                        }
                    }
                }
                stage('Client linux') {
                    agent {
                        dockerfile {
                            registryCredentialsId "${G_docker_creds}"
                            additionalBuildArgs "--pull --target ton-sdk-rust " + 
                                                "--build-arg \"TON_LABS_TYPES_IMAGE=${G_images['ton-labs-types']}\" " +
                                                "--build-arg \"TON_LABS_BLOCK_IMAGE=${G_images['ton-labs-block']}\" " + 
                                                "--build-arg \"TON_LABS_VM_IMAGE=${G_images['ton-labs-vm']}\" " + 
                                                "--build-arg \"TON_LABS_ABI_IMAGE=${G_images['ton-labs-abi']}\" " + 
                                                "--build-arg \"TON_EXECUTOR_IMAGE=${G_images['ton-executor']}\" " +
                                                "--build-arg \"TON_SDK_IMAGE=${G_images['ton-sdk']}\""
                        }
                    }
                    stages {
                        stage('Build') {
                            steps {
                                script {
                                    sh """
                                        cd /tonlabs/TON-SDK/ton_client/client
                                        node build.js
                                    """
                                }
                            }
                        }
                        stage('Deploy') {
                            when { 
                                expression {
                                    return GIT_BRANCH == 'master' || GIT_BRANCH ==~ '^PR-[0-9]+' || GIT_BRANCH == "${getVar(G_binversion)}-rc"
                                }
                            }
                            steps {
                                script {
                                    sh "cp /tonlabs/TON-SDK/ton_client/client/bin/*gz ./"
                                    withAWS(credentials: 'CI_bucket_writer', region: 'eu-central-1') {
                                        identity = awsIdentity()
                                        s3Upload \
                                            bucket: 'sdkbinaries.tonlabs.io', \
                                            includePathPattern:'*.gz', path: 'tmp_sdk', \
                                            workingDir:'.'
                                        }
                                }
                            }
                        }
                    }
					post {
						cleanup {script{cleanWs notFailBuild: true}}
					}
				}
                stage('Client macOS') {
                    agent {
                        label "ios"
                    }
                    stages {
                        stage('Report versions') {
                            steps {
                                sh 'rustc --version'
                                sh 'cargo --version'
                            }
                        }
                        stage('Get sources') {
                            steps {
                                script {
                                    def C_PATH = sh (script: 'pwd', returnStdout: true).trim()
                            
                                    unstash 'ton-sdk-src'
                                    sh """
                                        rm Cargo.toml
                                        unzip ton-sdk-src.zip
                                        node pathFix.js tonlabs/ton-labs-block/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/ton-labs-vm/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/ton-labs-abi/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/ton-executor/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/TON-SDK/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/TON-SDK/ton_sdk/Cargo.toml \"path = \\\"/tonlabs/\" \"path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/TON-SDK/ton_client/client/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/TON-SDK/ton_client/platforms/ton-client-node-js/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/TON-SDK/ton_client/platforms/ton-client-react-native/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/TON-SDK/ton_client/platforms/ton-client-web/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                    """
                                }
                            }
                        }
                        stage('Build') {
                            steps {
                                dir('tonlabs/TON-SDK/ton_client/client') {
                                    sshagent([G_gitcred]) {
                                        sh 'node build.js'
                                    }
                                }
                            }
                        }
                        stage('Deploy') {
                            when { 
                                expression {
                                    return GIT_BRANCH == 'master' || GIT_BRANCH ==~ '^PR-[0-9]+' || GIT_BRANCH == "${getVar(G_binversion)}-rc"
                                }
                            }
                            steps {
                                dir('tonlabs/TON-SDK/ton_client/client/bin') {
                                    script {
                                        withAWS(credentials: 'CI_bucket_writer', region: 'eu-central-1') {
                                            identity = awsIdentity()
                                            s3Upload \
                                                bucket: 'sdkbinaries.tonlabs.io', \
                                                includePathPattern:'*.gz', path: 'tmp_sdk', \
                                                workingDir:'.'
                                            }
                                    }
                                }
                            }
                        }
                    }
					post {
						cleanup {script{cleanWs notFailBuild: true}}
					}
				}
                stage('Client Windows') {
                    agent {
                        label "Win"
                    }
                    stages {
                        stage('Report versions') {
                            steps {
                                bat 'rustc --version'
                                bat 'cargo --version'
                            }
                        }
                        stage('Get sources') {
                            steps {
                                script {
                                    def C_PATH = bat (script: '@echo off && echo %cd%', returnStdout: true).trim()
                            
                                    unstash 'ton-sdk-src'
                                    bat """
                                        del Cargo.toml
                                        unzip ton-sdk-src.zip
                                        node pathFix.js tonlabs\\ton-labs-block\\Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}\\tonlabs\\\\\"
                                        node pathFix.js tonlabs\\ton-labs-vm\\Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}\\tonlabs\\\\\"
                                        node pathFix.js tonlabs\\ton-labs-abi\\Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}\\tonlabs\\\\\"
                                        node pathFix.js tonlabs\\ton-executor\\Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}\\tonlabs\\\\\"
                                        node pathFix.js tonlabs\\TON-SDK\\Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}\\tonlabs\\\\\"
                                        node pathFix.js tonlabs\\TON-SDK\\ton_sdk\\Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}\\tonlabs\\\\\"
                                        node pathFix.js tonlabs\\TON-SDK\\ton_sdk\\Cargo.toml \"/tonlabs/ton-executor\" \"${C_PATH}\\tonlabs\\ton-executor\\\\\"
                                        node pathFix.js tonlabs\\TON-SDK\\ton_client\\client\\Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}\\tonlabs\\\\\"
                                        node pathFix.js tonlabs\\TON-SDK\\ton_client\\platforms\\ton-client-node-js\\Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}\\tonlabs\\\\\"
                                        node pathFix.js tonlabs\\TON-SDK\\ton_client\\platforms\\ton-client-react-native\\Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}\\tonlabs\\\\\"
                                        node pathFix.js tonlabs\\TON-SDK\\ton_client\\platforms\\ton-client-web\\Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}\\tonlabs\\\\\"
                                    """
                                }
                            }
                        }
                        stage('Build') {
                            steps {
                                dir('tonlabs/TON-SDK/ton_client/client') {
                                    sshagent([G_gitcred]) {
                                        bat 'node build.js'
                                    }
                                }
                            }
                        }
                        stage('Deploy') {
                            when { 
                                expression {
                                    return GIT_BRANCH == 'master' || GIT_BRANCH ==~ '^PR-[0-9]+' || GIT_BRANCH == "${getVar(G_binversion)}-rc"
                                }
                            }
                            steps {
                                dir('tonlabs/TON-SDK/ton_client/client/bin') {
                                    script {
                                        withAWS(credentials: 'CI_bucket_writer', region: 'eu-central-1') {
                                            identity = awsIdentity()
                                            s3Upload \
                                                bucket: 'sdkbinaries.tonlabs.io', \
                                                includePathPattern:'*.gz', path: 'tmp_sdk', \
                                                workingDir:'.'
                                            }
                                    }
                                }
                            }
                        }
                    }
					post {
						cleanup {script{cleanWs notFailBuild: true}}
					}                
				}
                stage('react-native-ios') {
                    agent {
                        label "ios"
                    }
                    stages {
                        stage('Report versions') {
                            steps {
                                sh '''
                                rustc --version
                                cargo --version
                                '''
                            }
                        }
                        stage('Get sources') {
                            steps {
                                script {
                                    def C_PATH = sh (script: 'pwd', returnStdout: true).trim()
                            
                                    unstash 'ton-sdk-src'
                                    sh """
                                        rm Cargo.toml
                                        unzip ton-sdk-src.zip
                                        node pathFix.js tonlabs/ton-labs-block/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/ton-labs-vm/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/ton-labs-abi/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/ton-executor/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/TON-SDK/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/TON-SDK/ton_sdk/Cargo.toml \"path = \\\"/tonlabs/\" \"path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/TON-SDK/ton_client/client/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/TON-SDK/ton_client/platforms/ton-client-node-js/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/TON-SDK/ton_client/platforms/ton-client-react-native/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/TON-SDK/ton_client/platforms/ton-client-web/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                    """
                                }
                            }
                        }
                        stage('Build') {
                            steps {
                                echo 'Build ...'
                                sshagent([G_gitcred]) {
                                    dir('tonlabs/TON-SDK/ton_client/platforms/ton-client-react-native') {
                                        sh 'node build.js --ios'
                                    }
                                }
                            }
                            post {
                                failure {
                                    script { G_tsnj_build = false }
                                }
                            }
                        }
                        stage('Deploy') {
                            when { 
                                expression {
                                    return GIT_BRANCH == 'master' || GIT_BRANCH ==~ '^PR-[0-9]+' || GIT_BRANCH == "${getVar(G_binversion)}-rc"
                                }
                            }
                            steps {
                                dir('tonlabs/TON-SDK/ton_client/platforms/ton-client-react-native/output') {
                                    script {
                                        withAWS(credentials: 'CI_bucket_writer', region: 'eu-central-1') {
                                            identity = awsIdentity()
                                            s3Upload \
                                                bucket: 'sdkbinaries.tonlabs.io', \
                                                includePathPattern:'*.gz', path: 'tmp_sdk', \
                                                workingDir:'.'
                                            }
                                    }
                                }
                            }
                            post {
                                failure {
                                    script { G_tsnj_deploy = false }
                                }
                            }
                        }
                    }
					post {
						cleanup {script{cleanWs notFailBuild: true}}
					}                
				}
                stage('react-native-android') {
                    agent {
                        label "ios"
                    }
                    stages {
                        stage('Report versions') {
                            steps {
                                sh '''
                                rustc --version
                                cargo --version
                                '''
                            }
                        }
                        stage('Get sources') {
                            steps {
                                script {
                                    def C_PATH = sh (script: 'pwd', returnStdout: true).trim()
                            
                                    unstash 'ton-sdk-src'
                                    sh """
                                        rm Cargo.toml
                                        unzip ton-sdk-src.zip
                                        node pathFix.js tonlabs/ton-labs-block/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/ton-labs-vm/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/ton-labs-abi/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/ton-executor/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/TON-SDK/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/TON-SDK/ton_sdk/Cargo.toml \"path = \\\"/tonlabs/\" \"path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/TON-SDK/ton_client/client/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/TON-SDK/ton_client/platforms/ton-client-node-js/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/TON-SDK/ton_client/platforms/ton-client-react-native/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/TON-SDK/ton_client/platforms/ton-client-web/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                    """
                                }
                            }
                        }
                        stage('Build') {
                            steps {
                                echo 'Build ...'
                                sshagent([G_gitcred]) {
                                    dir('tonlabs/TON-SDK/ton_client/platforms/ton-client-react-native') {
                                        sh 'node build.js --android'
                                    }
                                }
                            }
                            post {
                                failure {
                                    script { G_tsnj_build = false }
                                }
                            }
                        }
                        stage('Deploy') {
                            when { 
                                expression {
                                    return GIT_BRANCH == 'master' || GIT_BRANCH ==~ '^PR-[0-9]+' || GIT_BRANCH == "${getVar(G_binversion)}-rc"
                                }
                            }
                            steps {
                                dir('tonlabs/TON-SDK/ton_client/platforms/ton-client-react-native/output') {
                                    script {
                                        withAWS(credentials: 'CI_bucket_writer', region: 'eu-central-1') {
                                            identity = awsIdentity()
                                            s3Upload \
                                                bucket: 'sdkbinaries.tonlabs.io', \
                                                includePathPattern:'*.gz', path: 'tmp_sdk', \
                                                workingDir:'.'
                                            }
                                    }
                                }
                            }
                            post {
                                failure {
                                    script { G_tsnj_deploy = false }
                                }
                            }
                        }
                    }
					post {
						cleanup {script{cleanWs notFailBuild: true}}
					}                
				}
                stage('node-js for iOS') {
                    agent {
                        label "ios"
                    }
                    stages {
                        stage('Report versions') {
                            steps {
                                sh '''
                                rustc --version
                                cargo --version
                                '''
                            }
                        }
                        stage('Get sources') {
                            steps {
                                script {
                                    def C_PATH = sh (script: 'pwd', returnStdout: true).trim()
                            
                                    unstash 'ton-sdk-src'
                                    sh """
                                        rm Cargo.toml
                                        unzip ton-sdk-src.zip
                                        node pathFix.js tonlabs/ton-labs-block/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/ton-labs-vm/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/ton-labs-abi/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/ton-executor/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/TON-SDK/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/TON-SDK/ton_sdk/Cargo.toml \"path = \\\"/tonlabs/\" \"path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/TON-SDK/ton_client/client/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/TON-SDK/ton_client/platforms/ton-client-node-js/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/TON-SDK/ton_client/platforms/ton-client-react-native/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                        node pathFix.js tonlabs/TON-SDK/ton_client/platforms/ton-client-web/Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}/tonlabs/\"
                                    """
                                }
                            }
                        }
                        stage('Build') {
                            steps {
                                echo 'Build ...'
                                sshagent([G_gitcred]) {
                                    dir('tonlabs/TON-SDK/ton_client/platforms/ton-client-node-js') {
                                        sh 'node build.js'
                                    }
                                }
                            }
                            post {
                                failure {
                                    script { G_tsnj_build = false }
                                }
                            }
                        }
                        stage('Deploy') {
                            when { 
                                expression {
                                    return GIT_BRANCH == 'master' || GIT_BRANCH ==~ '^PR-[0-9]+' || GIT_BRANCH == "${getVar(G_binversion)}-rc"
                                }
                            }
                            steps {
                                dir('tonlabs/TON-SDK/ton_client/platforms/ton-client-node-js/bin') {
                                    script {
                                        withAWS(credentials: 'CI_bucket_writer', region: 'eu-central-1') {
                                            identity = awsIdentity()
                                            s3Upload \
                                                bucket: 'sdkbinaries.tonlabs.io', \
                                                includePathPattern:'*.gz', path: 'tmp_sdk', \
                                                workingDir:'.'
                                            }
                                    }
                                }
                            }
                            post {
                                failure {
                                    script { G_tsnj_deploy = false }
                                }
                            }
                        }
                    }
					post {
						cleanup {script{cleanWs notFailBuild: true}}
					}                
				}
                stage('node-js for Windows') {
                    agent {
                        label "Win"
                    }
                    stages {
                        stage('Report versions') {
                            steps {
                                bat '''
                                rustc --version
                                cargo --version
                                '''
                            }
                        }
                        stage('Get sources') {
                            steps {
                                script {
                                    def C_PATH = bat (script: '@echo off && echo %cd%', returnStdout: true).trim()
                            
                                    unstash 'ton-sdk-src'
                                    bat """
                                        del Cargo.toml
                                        unzip ton-sdk-src.zip

                                        node pathFix.js tonlabs\\ton-labs-block\\Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}\\tonlabs\\\\\"
                                        node pathFix.js tonlabs\\ton-labs-vm\\Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}\\tonlabs\\\\\"
                                        node pathFix.js tonlabs\\ton-labs-abi\\Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}\\tonlabs\\\\\"
                                        node pathFix.js tonlabs\\ton-executor\\Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}\\tonlabs\\\\\"
                                        node pathFix.js tonlabs\\TON-SDK\\Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}\\tonlabs\\\\\"
                                        node pathFix.js tonlabs\\TON-SDK\\ton_sdk\\Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}\\tonlabs\\\\\"
                                        node pathFix.js tonlabs\\TON-SDK\\ton_sdk\\Cargo.toml \"/tonlabs/ton-executor\" \"${C_PATH}\\tonlabs\\ton-executor\\\\\"
                                        node pathFix.js tonlabs\\TON-SDK\\ton_client\\client\\Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}\\tonlabs\\\\\"
                                        node pathFix.js tonlabs\\TON-SDK\\ton_client\\platforms\\ton-client-node-js\\Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}\\tonlabs\\\\\"
                                        node pathFix.js tonlabs\\TON-SDK\\ton_client\\platforms\\ton-client-react-native\\Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}\\tonlabs\\\\\"
                                        node pathFix.js tonlabs\\TON-SDK\\ton_client\\platforms\\ton-client-web\\Cargo.toml \"{ path = \\\"/tonlabs/\" \"{ path = \\\"${C_PATH}\\tonlabs\\\\\"
                                    """
                                }
                            }
                        }
                        stage('Build') {
                            steps {
                                echo 'Build ...'
                                sshagent([G_gitcred]) {
                                    dir('tonlabs/TON-SDK/ton_client/platforms/ton-client-node-js') {
                                        bat 'node build.js'
                                    }
                                }
                            }
                            post {
                                failure {
                                    script { G_tsnj_build = false }
                                }
                            }
                        }
                        stage('Deploy') {
                            when { 
                                expression {
                                    return GIT_BRANCH == 'master' || GIT_BRANCH ==~ '^PR-[0-9]+' || GIT_BRANCH == "${getVar(G_binversion)}-rc"
                                }
                            }
                            steps {
                                dir('tonlabs/TON-SDK/ton_client/platforms/ton-client-node-js/bin') {
                                    script {
                                        withAWS(credentials: 'CI_bucket_writer', region: 'eu-central-1') {
                                            identity = awsIdentity()
                                            s3Upload \
                                                bucket: 'sdkbinaries.tonlabs.io', \
                                                includePathPattern:'*.gz', path: 'tmp_sdk', \
                                                workingDir:'.'
                                            }
                                    }
                                }
                            }
                            post {
                                failure {
                                    script { G_tsnj_deploy = false }
                                }
                            }
                        }
                    }
					post {
						cleanup {script{cleanWs notFailBuild: true}}
					}                
				}
                stage('node-js for Linux') {
                    agent {
                        dockerfile {
                            registryCredentialsId "${G_docker_creds}"
                            additionalBuildArgs "--pull --target ton-sdk-rust " + 
                                                "--build-arg \"TON_LABS_TYPES_IMAGE=${G_images['ton-labs-types']}\" " +
                                                "--build-arg \"TON_LABS_BLOCK_IMAGE=${G_images['ton-labs-block']}\" " + 
                                                "--build-arg \"TON_LABS_VM_IMAGE=${G_images['ton-labs-vm']}\" " + 
                                                "--build-arg \"TON_LABS_ABI_IMAGE=${G_images['ton-labs-abi']}\" " + 
                                                "--build-arg \"TON_EXECUTOR_IMAGE=${G_images['ton-executor']}\" " +
                                                "--build-arg \"TON_SDK_IMAGE=${G_images['ton-sdk']}\""
                        }
                    }
                    stages {
                        stage('Build') {
                            steps {
                                sshagent([G_gitcred]) {
                                    sh """
                                        cd /tonlabs/TON-SDK/ton_client/platforms/ton-client-node-js
                                        node build.js
                                    """
                                }
                            }
                            post {
                                failure {
                                    script { G_tsnj_build = false }
                                }
                            }
                        }
                        stage('Deploy') {
                            when { 
                                expression {
                                    return GIT_BRANCH == 'master' || GIT_BRANCH ==~ '^PR-[0-9]+' || GIT_BRANCH == "${getVar(G_binversion)}-rc"
                                }
                            }
                            steps {
                                script {
                                    sh "cp /tonlabs/TON-SDK/ton_client/platforms/ton-client-node-js/bin/*.gz ./"
                                    withAWS(credentials: 'CI_bucket_writer', region: 'eu-central-1') {
                                        identity = awsIdentity()
                                        s3Upload \
                                            bucket: 'sdkbinaries.tonlabs.io', \
                                            includePathPattern:'*.gz', path: 'tmp_sdk', \
                                            workingDir:'.'
                                        }
                                }
                            }
                            post {
                                failure {
                                    script { G_tsnj_deploy = false }
                                }
                            }
                        }
                    }
					post {
						cleanup {script{cleanWs notFailBuild: true}}
					}                
				}
                stage('web') {
                    agent {
                        dockerfile {
                            registryCredentialsId "${G_docker_creds}"
                            additionalBuildArgs "--pull --target ton-sdk-rust " + 
                                                "--build-arg \"TON_LABS_TYPES_IMAGE=${G_images['ton-labs-types']}\" " +
                                                "--build-arg \"TON_LABS_BLOCK_IMAGE=${G_images['ton-labs-block']}\" " + 
                                                "--build-arg \"TON_LABS_VM_IMAGE=${G_images['ton-labs-vm']}\" " + 
                                                "--build-arg \"TON_LABS_ABI_IMAGE=${G_images['ton-labs-abi']}\" " + 
                                                "--build-arg \"TON_EXECUTOR_IMAGE=${G_images['ton-executor']}\" " +
                                                "--build-arg \"TON_SDK_IMAGE=${G_images['ton-sdk']}\""
                        }
                    }
                    stages {
                        stage('Build') {
                            environment {
                                X86_64_UNKNOWN_LINUX_GNU_OPENSSL_LIB_DIR='/usr/lib/x86_64-linux-gnu'
                                X86_64_UNKNOWN_LINUX_GNU_OPENSSL_INCLUDE_DIR='/usr/include/openssl'
                                X86_64_UNKNOWN_LINUX_GNU_OPENSSL_DIR='/usr/bin/openssl'
                                OPENSSL_DIR='/usr/bin/openssl'
                            }
                            steps {
                                sshagent([G_gitcred]) {
                                    sh """
                                        cd /tonlabs/TON-SDK/ton_client/platforms/ton-client-web
                                        npm install
                                        node build.js
                                    """
                                }
                            }
                            post {
                                failure {
                                    script { G_tsnj_build = false }
                                }
                            }
                        }
                        stage('Deploy') {
                            when { 
                                expression {
                                    return GIT_BRANCH == 'master' || GIT_BRANCH ==~ '^PR-[0-9]+' || GIT_BRANCH == "${getVar(G_binversion)}-rc"
                                }
                            }
                            steps {
                                script {
                                    sh "cp /tonlabs/TON-SDK/ton_client/platforms/ton-client-web/bin/*.gz ./"
                                    withAWS(credentials: 'CI_bucket_writer', region: 'eu-central-1') {
                                        identity = awsIdentity()
                                        s3Upload \
                                            bucket: 'sdkbinaries.tonlabs.io', \
                                            includePathPattern:'*.gz', path: 'tmp_sdk', \
                                            workingDir:'.'
                                    }
                                }
                            }
                            post {
                                failure {
                                    script { G_tsnj_deploy = false }
                                }
                            }
                        }
                    }
					post {
						cleanup {script{cleanWs notFailBuild: true}}
					}                
				}
            }
        }
        stage('After stages') {
            when {
                expression {
                    return !isUpstream() && (GIT_BRANCH == 'master' || GIT_BRANCH ==~ '^PR-[0-9]+' || GIT_BRANCH == "${getVar(G_binversion)}-rc")
                }
            }
            steps {
                script {
                    def afterParams = G_params
                    afterParams.push(string("name": "project_name", "value": "ton-sdk"))
                    afterParams.push(string("name": "stage", "value": "after"))
                    build job: 'Builder/build-flow', parameters: afterParams
                }
            }
        }
    }
    post {
        failure {
            script {
                withAWS(credentials: 'CI_bucket_writer', region: 'eu-central-1') {
                    identity = awsIdentity()
                    s3Delete bucket: 'sdkbinaries.tonlabs.io', path: 'tmp_sdk/'
                }
                def cause = "${currentBuild.getBuildCauses()}"
                echo "${cause}"
                if(!cause.matches('upstream')) {
                    withAWS(credentials: 'CI_bucket_writer', region: 'eu-central-1') {
                        identity = awsIdentity()
                        s3Download bucket: 'sdkbinaries.tonlabs.io', file: 'version.json', force: true, path: 'version.json'
                    }
                    sh """
                        echo const fs = require\\(\\'fs\\'\\)\\; > decline.js
                        echo const ver = JSON.parse\\(fs.readFileSync\\(\\'version.json\\'\\, \\'utf8\\'\\)\\)\\; >> decline.js
                        echo if\\(!ver.release\\) { throw new Error\\(\\'Unable to set decline version\\'\\)\\; } >> decline.js
                        echo ver.candidate = \\'\\'\\; >> decline.js
                        echo fs.writeFileSync\\(\\'version.json\\', JSON.stringify\\(ver\\)\\)\\; >> decline.js
                        cat decline.js
                        cat version.json
                        node decline.js
                    """
                    withAWS(credentials: 'CI_bucket_writer', region: 'eu-central-1') {
                        identity = awsIdentity()
                        s3Upload \
                            bucket: 'sdkbinaries.tonlabs.io', \
                            includePathPattern:'version.json', workingDir:'.'
                    }
                }
            }
        }
    }
}