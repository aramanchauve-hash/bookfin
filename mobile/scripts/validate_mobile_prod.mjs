import https from 'https';
import http from 'http';
import fs from 'fs';
import path from 'path';
import crypto from 'crypto';

const API_BASE = 'https://bookfin-api-production.up.railway.app';

function request(url, options = {}, body = null) {
  return new Promise((resolve, reject) => {
    const parsedUrl = new URL(url);
    const transport = parsedUrl.protocol === 'https:' ? https : http;
    const reqOptions = {
      method: options.method || 'GET',
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json',
        'X-App-Version': '1.0.0',
        'X-Build-Version': '1',
        ...(options.headers || {}),
      },
    };

    const req = transport.request(url, reqOptions, (res) => {
      let data = '';
      res.on('data', (chunk) => { data += chunk; });
      res.on('end', () => {
        try {
          const parsed = data ? JSON.parse(data) : {};
          resolve({ status: res.statusCode, headers: res.headers, data: parsed });
        } catch {
          resolve({ status: res.statusCode, headers: res.headers, data });
        }
      });
    });

    req.on('error', reject);
    req.on('timeout', () => { req.destroy(); reject(new Error('Timeout')); });
    if (body) {
      req.write(typeof body === 'string' ? body : JSON.stringify(body));
    }
    req.end();
  });
}

async function runValidation() {
  console.log('==================================================');
  console.log('BOOKFIN — LIVE RAILWAY PRODUCTION VALIDATION');
  console.log('API Base:', API_BASE);
  console.log('==================================================\n');

  const report = {
    health: false,
    onboarding: false,
    frFeed: false,
    enFeed: false,
    esFeed: false,
    reactions: false,
    reveals: false,
    continuations: false,
    stressTest30: false,
    validation422: false,
    contentV2Strict: false,
    reviewWorks: {},
  };

  // 1. Health check
  console.log('1. Testing /health...');
  const healthRes = await request(`${API_BASE}/health`);
  if (healthRes.status === 200) {
    console.log('   ✓ /health returned 200 OK');
    report.health = true;
  } else {
    console.error('   ✗ /health failed:', healthRes.status, healthRes.data);
  }

  // 2. Anonymous Identity & Onboarding Language Sync
  console.log('\n2. Testing Anonymous Identity creation and Language Sync...');
  const idRes = await request(`${API_BASE}/api/v1/identities/anonymous`, { method: 'POST' }, {});
  if ((idRes.status === 200 || idRes.status === 201) && idRes.data.user_id) {
    const userId = idRes.data.user_id;
    console.log('   ✓ Created anonymous user:', userId);

    // Sync FR, EN, ES
    const langRes = await request(`${API_BASE}/api/v1/users/${userId}/languages`, { method: 'PUT' }, {
      language_tags: ['fr', 'en', 'es']
    });
    if ((langRes.status === 200 || langRes.status === 201) && Array.isArray(langRes.data.language_tags) && langRes.data.language_tags.length === 3) {
      console.log('   ✓ Synced language preferences:', langRes.data.language_tags.join(', '));
      report.onboarding = true;
    } else {
      console.error('   ✗ Language sync failed:', langRes.status, langRes.data);
    }
  } else {
    console.error('   ✗ Anonymous identity creation failed:', idRes.status, idRes.data);
  }

  // 3. Language Feeds & Content V2 Validation
  console.log('\n3. Testing Language Feeds (FR, EN, ES) with Content V2 and Hashes...');
  const languages = ['fr', 'en', 'es'];
  const feedPages = {};

  for (const lang of languages) {
    const userRes = await request(`${API_BASE}/api/v1/identities/anonymous`, { method: 'POST' }, {});
    const uId = userRes.data.user_id;
    await request(`${API_BASE}/api/v1/users/${uId}/languages`, { method: 'PUT' }, { language_tags: [lang] });

    console.log(`   Fetching ${lang.toUpperCase()} feed page...`);
    const pageRes = await request(`${API_BASE}/api/v1/feed/next?user_id=${uId}`);
    if (pageRes.status === 200 && pageRes.data.page_id) {
      const p = pageRes.data;
      feedPages[lang] = p;
      const hasBlocks = Array.isArray(p.blocks) && p.blocks.length > 0;
      const hasHash = typeof p.content_hash === 'string' && p.content_hash.length === 64;
      const correctLang = p.language_tag === lang;

      console.log(`     Page ID: ${p.page_id}`);
      console.log(`     Language: ${p.language_tag} (expected: ${lang})`);
      console.log(`     Content Hash: ${p.content_hash}`);
      console.log(`     Blocks Count: ${p.blocks ? p.blocks.length : 0}`);
      console.log(`     First Block Type: ${hasBlocks ? p.blocks[0].type : 'NONE'}`);

      if (hasBlocks && hasHash && correctLang) {
        console.log(`     ✓ ${lang.toUpperCase()} Feed page verified valid V2`);
        if (lang === 'fr') report.frFeed = true;
        if (lang === 'en') report.enFeed = true;
        if (lang === 'es') report.esFeed = true;
      } else {
        console.error(`     ✗ ${lang.toUpperCase()} Feed page failed checks`);
      }
    } else {
      console.error(`     ✗ Failed to fetch ${lang.toUpperCase()} feed:`, pageRes.status, pageRes.data);
    }
  }

  // 4. Reaction, Reveal, and Continuation Flow
  console.log('\n4. Testing Reaction, Reveal, and Continuation Flow...');
  const testUserRes = await request(`${API_BASE}/api/v1/identities/anonymous`, { method: 'POST' }, {});
  const testUserId = testUserRes.data.user_id;
  await request(`${API_BASE}/api/v1/users/${testUserId}/languages`, { method: 'PUT' }, { language_tags: ['fr'] });

  const p1Res = await request(`${API_BASE}/api/v1/feed/next?user_id=${testUserId}`);
  const p1 = p1Res.data;
  console.log('   Fetched Page 1:', p1.page_id);

  // Check reveal before reaction (must be rejected)
  const preReveal = await request(`${API_BASE}/api/v1/pages/${p1.page_id}/reveal?user_id=${testUserId}`);
  if (preReveal.status === 403 || (typeof preReveal.data === 'string' && preReveal.data.includes('réaction préalable'))) {
    console.log('   ✓ Reveal before reaction correctly rejected');
  } else {
    console.warn('   ! Reveal before reaction response:', preReveal.status, preReveal.data);
  }

  // Check 422 validation rule (fast reaction rejected with 422)
  console.log('   Testing 422 validation rule (immediate reaction)...');
  const fastReact = await request(`${API_BASE}/api/v1/reactions`, { method: 'POST' }, {
    user_id: testUserId,
    page_id: p1.page_id,
    impression_id: p1.impression_id || p1.id,
    reaction: 'like',
    reading_time_ms: 200,
    scroll_depth: 0.1,
    bottom_reached: false,
    content_overflows: true,
    event_id: crypto.randomUUID(),
    served_at: p1.served_at,
  });
  if (fastReact.status === 422 && fastReact.data.error === 'reading_validation_failed') {
    console.log(`   ✓ 422 reading validation properly triggered (reason: ${fastReact.data.reason})`);
    report.validation422 = true;
  } else {
    console.warn('   ! Fast reaction status:', fastReact.status, fastReact.data);
  }

  // Submit reaction with proper delay
  console.log('   Simulating active reading (waiting 4s)...');
  await new Promise((r) => setTimeout(r, 4200));

  const eventId = crypto.randomUUID();
  const reactRes = await request(`${API_BASE}/api/v1/reactions`, { method: 'POST' }, {
    user_id: testUserId,
    page_id: p1.page_id,
    impression_id: p1.impression_id || p1.id,
    reaction: 'like',
    reading_time_ms: 4500,
    scroll_depth: 1.0,
    bottom_reached: true,
    content_overflows: false,
    event_id: eventId,
    served_at: p1.served_at,
  });

  if ((reactRes.status === 200 || reactRes.status === 201) && reactRes.data.success) {
    console.log('   ✓ Reaction submitted successfully:', reactRes.data.reaction_id);
    report.reactions = true;

    // Reveal metadata post-reaction
    const postReveal = await request(`${API_BASE}/api/v1/pages/${p1.page_id}/reveal?user_id=${testUserId}`);
    if (postReveal.status === 200 && postReveal.data.title && postReveal.data.author) {
      console.log(`   ✓ Metadata revealed: "${postReveal.data.title}" by ${postReveal.data.author} (${postReveal.data.source_name})`);
      report.reveals = true;
    } else {
      console.error('   ✗ Post-reaction reveal failed:', postReveal.status, postReveal.data);
    }

    // Continue reading
    const contRes = await request(`${API_BASE}/api/v1/pages/${p1.page_id}/continue`, { method: 'POST' }, {
      user_id: testUserId,
      parent_impression_id: p1.impression_id || p1.id,
      client_request_id: crypto.randomUUID(),
    });

    if (contRes.status === 200 && contRes.data.page_id) {
      console.log(`   ✓ Continuation page fetched: sequence ${contRes.data.page_sequence_number}, depth ${contRes.data.continuation_depth}`);
      report.continuations = true;
    } else {
      console.error('   ✗ Continuation failed:', contRes.status, contRes.data);
    }
  } else {
    console.error('   ✗ Reaction submission failed:', reactRes.status, reactRes.data);
  }

  // 5. Stress Test: 30 consecutive page navigations against Railway production
  console.log('\n5. Running 30-Page Stress Test across FR, EN, ES against Railway...');
  const stressUserRes = await request(`${API_BASE}/api/v1/identities/anonymous`, { method: 'POST' }, {});
  const stressUserId = stressUserRes.data.user_id;
  await request(`${API_BASE}/api/v1/users/${stressUserId}/languages`, { method: 'PUT' }, { language_tags: ['fr', 'en', 'es'] });

  let consecutiveSuccesses = 0;
  let allV2Valid = true;
  let allHashesValid = true;
  const sampledWorks = new Set();

  for (let i = 1; i <= 30; i++) {
    const res = await request(`${API_BASE}/api/v1/feed/next?user_id=${stressUserId}`);
    if (res.status === 200 && res.data.page_id) {
      consecutiveSuccesses += 1;
      const p = res.data;
      if (!Array.isArray(p.blocks) || p.blocks.length === 0) allV2Valid = false;
      if (!p.content_hash || p.content_hash.length !== 64) allHashesValid = false;
      process.stdout.write(`\r   Page ${i}/30: [${p.language_tag.toUpperCase()}] hash: ${p.content_hash.slice(0, 10)}... blocks: ${p.blocks ? p.blocks.length : 0}`);
    } else {
      console.error(`\n   ✗ Page ${i} failed:`, res.status, res.data);
      break;
    }
  }
  console.log('');

  if (consecutiveSuccesses === 30 && allV2Valid && allHashesValid) {
    console.log('   ✓ 30/30 consecutive pages loaded successfully with valid Content V2 and hashes');
    report.stressTest30 = true;
    report.contentV2Strict = true;
  } else {
    console.error(`   ✗ Stress test failed: ${consecutiveSuccesses}/30 succeeded, V2 valid: ${allV2Valid}, hashes: ${allHashesValid}`);
  }

  // 6. Verification of the 3 REVIEW works
  console.log('\n6. Inspecting the 3 REVIEW works in Curated V1...');
  const reviewWorkIds = [
    'fr-barbey-les-diaboliques',
    'es-unamuno-niebla',
    'es-valle-inclan-luces-de-bohemia',
  ];

  for (const wId of reviewWorkIds) {
    const lang = wId.startsWith('fr') ? 'fr' : 'es';
    const filePath = path.resolve(process.cwd(), `../corpus/curated_v1/pages/${lang}/${wId}_pages.json`);
    if (fs.existsSync(filePath)) {
      const content = JSON.parse(fs.readFileSync(filePath, 'utf8'));
      const samplePage = content.pages[0];
      const hasBlocks = Array.isArray(samplePage.blocks) && samplePage.blocks.length > 0;
      const hasHash = typeof samplePage.content_hash === 'string' && samplePage.content_hash.length === 64;
      const blockTypes = new Set(samplePage.blocks.map(b => b.type));

      console.log(`   Work: ${wId}`);
      console.log(`     Pages count: ${content.pages_count || content.pages.length}`);
      console.log(`     Sample page sequence: ${samplePage.page_sequence_number}`);
      console.log(`     Has valid blocks: ${hasBlocks} (${Array.from(blockTypes).join(', ')})`);
      console.log(`     Content hash: ${samplePage.content_hash}`);

      report.reviewWorks[wId] = {
        status: hasBlocks && hasHash ? 'VISUALLY ACCEPTABLE' : 'NEEDS REVIEW',
        blocks: Array.from(blockTypes),
        hash: samplePage.content_hash,
      };
      console.log(`     Recommendation: ${report.reviewWorks[wId].status}`);
    } else {
      console.error(`   ✗ Work file not found: ${filePath}`);
    }
  }

  console.log('\n==================================================');
  console.log('VALIDATION SUMMARY');
  console.log('==================================================');
  console.log(JSON.stringify(report, null, 2));

  return report;
}

runValidation().catch(console.error);
