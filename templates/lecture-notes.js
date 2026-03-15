(() => {
  const tocList = document.querySelector('[data-lecture-notes="toc"]');
  const sectionsContainer = document.querySelector('[data-lecture-notes="sections"]');
  const statusEl = document.querySelector('[data-lecture-notes="status"]');
  const tocShell = document.querySelector('[data-toc-shell]');
  const tocToggle = document.querySelector('[data-toc-toggle]');
  const currentSubjectEl = document.querySelector('[data-lecture-notes="current-subject"]');
  const reduceMotionQuery = window.matchMedia('(prefers-reduced-motion: reduce)');
  const desktopTocModeQuery = window.matchMedia('(min-width: 981px)');

  let sectionTrackingEntries = [];
  let sectionObserver = null;
  let tocCompactThreshold = Number.POSITIVE_INFINITY;
  let tocPlaceholder = null;

  if (!tocList || !sectionsContainer || !tocShell) {
    return;
  }

  const setStatus = (message, isError) => {
    if (!statusEl) {
      return;
    }

    statusEl.textContent = message;
    statusEl.classList.toggle('error', Boolean(isError));
    statusEl.hidden = !message;
  };

  const setTocMenuOpen = (isOpen) => {
    tocShell.setAttribute('data-menu-open', String(isOpen));

    if (!tocToggle) {
      return;
    }

    tocToggle.setAttribute('aria-expanded', String(isOpen));
    tocToggle.setAttribute('aria-label', isOpen ? 'Close subjects list' : 'Open subjects list');

    const icon = tocToggle.querySelector('i');
    if (icon) {
      icon.classList.toggle('fa-bars', !isOpen);
      icon.classList.toggle('fa-xmark', isOpen);
    }
  };

  const closeTocMenu = () => {
    setTocMenuOpen(false);
  };

  const ensureTocPlaceholder = (height) => {
    if (tocPlaceholder) {
      tocPlaceholder.style.height = `${height}px`;
      return;
    }

    tocPlaceholder = document.createElement('div');
    tocPlaceholder.className = 'toc-compact-placeholder';
    tocPlaceholder.style.height = `${height}px`;
    tocShell.parentNode.insertBefore(tocPlaceholder, tocShell);
  };

  const removeTocPlaceholder = () => {
    if (!tocPlaceholder) {
      return;
    }

    tocPlaceholder.remove();
    tocPlaceholder = null;
  };

  const setCompactMode = (isCompact) => {
    const currentlyCompact = tocShell.classList.contains('is-compact');
    if (currentlyCompact === isCompact) {
      return;
    }

    if (isCompact) {
      ensureTocPlaceholder(tocShell.getBoundingClientRect().height);
      tocShell.classList.add('is-compact');
      closeTocMenu();
      return;
    }

    tocShell.classList.remove('is-compact');
    removeTocPlaceholder();
    closeTocMenu();
  };

  const recalcTocCompactThreshold = () => {
    if (!desktopTocModeQuery.matches) {
      tocCompactThreshold = Number.POSITIVE_INFINITY;
      return;
    }

    const thresholdTarget = tocPlaceholder || tocShell;
    const rect = thresholdTarget.getBoundingClientRect();
    tocCompactThreshold = window.scrollY + rect.bottom;
  };

  const syncTocDisplayMode = () => {
    if (!desktopTocModeQuery.matches) {
      setCompactMode(false);
      return;
    }

    const shouldUseCompact = window.scrollY >= tocCompactThreshold;
    setCompactMode(shouldUseCompact);
  };

  const setActiveTocButton = (targetButton) => {
    const tocButtons = tocList.querySelectorAll('button');
    tocButtons.forEach((button) => {
      const isActive = button === targetButton;
      button.classList.toggle('is-active', isActive);
      button.setAttribute('aria-current', isActive ? 'page' : 'false');
    });

    if (currentSubjectEl) {
      currentSubjectEl.textContent = targetButton ? targetButton.textContent : 'Subjects';
    }
  };

  const updateActiveTocFromScroll = () => {
    if (!sectionTrackingEntries.length) {
      return;
    }

    const markerY = window.scrollY + window.innerHeight * 0.34;
    let activeEntry = sectionTrackingEntries[0];

    sectionTrackingEntries.forEach((entry) => {
      const sectionTop = entry.sectionEl.getBoundingClientRect().top + window.scrollY;
      if (sectionTop <= markerY) {
        activeEntry = entry;
      }
    });

    setActiveTocButton(activeEntry.button);
  };

  const setupSectionTracking = (entries) => {
    sectionTrackingEntries = entries;

    if (sectionObserver) {
      sectionObserver.disconnect();
      sectionObserver = null;
    }

    if (!sectionTrackingEntries.length) {
      return;
    }

    if ('IntersectionObserver' in window) {
      sectionObserver = new IntersectionObserver(
        () => {
          updateActiveTocFromScroll();
        },
        {
          root: null,
          threshold: [0, 0.2, 0.4, 0.8],
          rootMargin: '-20% 0px -55% 0px',
        }
      );

      sectionTrackingEntries.forEach((entry) => {
        sectionObserver.observe(entry.sectionEl);
      });
    }

    updateActiveTocFromScroll();
  };

  const bindTocEntries = () => {
    const tocButtons = [...tocList.querySelectorAll('button[data-target-id]')];
    const trackingEntries = [];

    tocButtons.forEach((button) => {
      const targetId = button.getAttribute('data-target-id');
      if (!targetId) {
        return;
      }

      const sectionEl = document.getElementById(targetId);
      if (!sectionEl) {
        return;
      }

      button.addEventListener('click', () => {
        sectionEl.scrollIntoView({
          behavior: reduceMotionQuery.matches ? 'auto' : 'smooth',
          block: 'start',
        });
        setActiveTocButton(button);
        closeTocMenu();
      });

      trackingEntries.push({ sectionEl, button });
    });

    if (trackingEntries.length) {
      setActiveTocButton(trackingEntries[0].button);
    } else {
      setActiveTocButton(null);
    }

    setupSectionTracking(trackingEntries);
  };

  if (!sectionsContainer.children.length) {
    setStatus('No lecture notes published yet.', false);
  } else {
    setStatus('', false);
  }

  if (tocToggle) {
    tocToggle.addEventListener('click', () => {
      const isOpen = tocShell.getAttribute('data-menu-open') === 'true';
      setTocMenuOpen(!isOpen);
    });
  }

  setTocMenuOpen(false);
  bindTocEntries();

  document.addEventListener('click', (event) => {
    if (!tocShell.contains(event.target)) {
      closeTocMenu();
    }
  });

  document.addEventListener('keydown', (event) => {
    if (event.key === 'Escape') {
      closeTocMenu();
    }
  });

  window.addEventListener(
    'scroll',
    () => {
      syncTocDisplayMode();
      updateActiveTocFromScroll();
    },
    { passive: true }
  );

  window.addEventListener('resize', () => {
    recalcTocCompactThreshold();
    syncTocDisplayMode();
    updateActiveTocFromScroll();
  });

  desktopTocModeQuery.addEventListener('change', () => {
    recalcTocCompactThreshold();
    syncTocDisplayMode();
  });

  recalcTocCompactThreshold();
  syncTocDisplayMode();
})();
