(function () {
    async function loadDiff(details) {
        const body = details.querySelector('.diff-file-body');
        const url = details.dataset.diffUrl;

        if (!body || !url) {
            return;
        }

        if (details.dataset.loading === 'true') {
            return;
        }

        if (details.dataset.loaded === 'true') {
            return;
        }

        details.dataset.loading = 'true';
        body.innerHTML = '';

        try {
            const response = await fetch(url, {
                headers: {
                    'X-Requested-With': 'fetch',
                },
            });

            if (!response.ok) {
                throw new Error(String(response.status));
            }

            body.innerHTML = await response.text();
            details.dataset.loaded = 'true';
            delete details.dataset.error;
        } catch (_error) {
            details.dataset.error = 'true';
            body.innerHTML = '<div class="diff-file-state diff-file-error">Could not load diff.</div>';
        } finally {
            delete details.dataset.loading;
        }
    }

    function bindDiff(details) {
        details.addEventListener('toggle', function () {
            if (!details.open) {
                return;
            }

            loadDiff(details);
        });
    }

    document.addEventListener('DOMContentLoaded', function () {
        document.querySelectorAll('.diff-file[data-diff-url]').forEach(function (details) {
            bindDiff(details);
        });
    });
})();
