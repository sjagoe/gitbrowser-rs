module.exports = async ({github, context, core}) => {
  const pullRequests = await github.rest.repos.listPullRequestsAssociatedWithCommit({
    commit_sha: context.sha,
    owner: context.repo.owner,
    repo: context.repo.repo,
  });

  const noRelease = 'no release';

  if (pullRequests.data.length === 0) {
    core.setOutput('release', 'false');
    core.setOutput('release-kind', noRelease);
  } else if (pullRequests.data.length > 1) {
    core.error(`Expected zero or one pull request for commit, got ${pullRequests.data.length}`);
    throw new Error('Unexpected pull request data');
  } else {
    const pullRequest = pullRequests.data[0];

    const labels = pullRequest.labels.map(
      (l) => l.name).filter(
        (label) => ['major', 'minor', 'patch', 'no release'].includes(label));

    if (labels.length > 1) {
      core.error('Expected only one label of "major", "minor", "patch"');
    }

    const label = labels.length > 0 ? labels[0] : noRelease;

    core.setOutput('release', (label !== noRelease).toString());
    core.setOutput('release-kind', label);
  }
};
