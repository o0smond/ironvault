// Cloudflare Worker: index.js
// Handles DB actions for Rust client via HTTP POST
// Supports: fetch, fetch_by_id, insert, update, update_by_column, delete, fetch_by_user

const corsHeaders = {
  'Access-Control-Allow-Origin': '*',
  'Access-Control-Allow-Methods': 'POST, OPTIONS',
  'Access-Control-Allow-Headers': 'Content-Type',
};

export default {
  async fetch(request, env) {
    try {
      return await handle_request(request, env);
    } catch (err) {
      return new Response(`Worker error: ${err.message}`, { status: 500, headers: corsHeaders });
    }
  }
};

async function handle_request(request, env) {
  if (request.method === 'OPTIONS') {
    return new Response(null, { status: 204, headers: corsHeaders });
  }

  if (request.method !== 'POST') {
    return new Response('Method Not Allowed', { status: 405, headers: corsHeaders });
  }

  let data;
  try {
    data = await request.json();
  } catch {
    return new Response('Invalid JSON', { status: 400, headers: corsHeaders });
  }

  const { action, table, id, record, column, value, username } = data;

  if (!action || !table) {
    return new Response('Missing action or table', { status: 400, headers: corsHeaders });
  }

  let safeTable;
  try {
    safeTable = safe_identifier(table);
  } catch {
    return new Response('Invalid table name', { status: 400, headers: corsHeaders });
  }

  let safeColumn = null;
  if (column) {
    try {
      safeColumn = safe_identifier(column);
    } catch {
      return new Response('Invalid column name', { status: 400, headers: corsHeaders });
    }
  }

  let sql;
  let params = [];

  switch (action) {
    case 'fetch':
      sql = `SELECT * FROM ${safeTable}`;
      break;

    case 'fetch_by_id':
      sql = `SELECT * FROM ${safeTable} WHERE id = ?`;
      params = [wrap_param(id)];
      break;

    case 'fetch_by_user':
      if (!username) {
        return new Response('Missing username', { status: 400, headers: corsHeaders });
      }
      sql = `SELECT id FROM ${safeTable} WHERE username = ?`;
      params = [wrap_param(username)];
      break;

    case 'insert': {
      if (!record) {
        return new Response('Missing record', { status: 400, headers: corsHeaders });
      }
      if ('username' in record && !record.username) {
        return new Response('Missing username', { status: 400, headers: corsHeaders });
      }

      const keys = Object.keys(record).filter(k => k !== 'id');
      const cols = keys.map(k => safe_identifier(k)).join(', ');
      const placeholders = keys.map(() => '?').join(', ');

      sql = `INSERT INTO ${safeTable} (${cols}) VALUES (${placeholders})`;
      params = keys.map(k => wrap_param(record[k]));
      break;
    }

    case 'update': {
      if (!record || record.id == null) {
        return new Response('Missing record or id', { status: 400, headers: corsHeaders });
      }

      const updateKeys = Object.keys(record).filter(k => k !== 'id');
      const setClause = updateKeys.map(k => `${safe_identifier(k)} = ?`).join(', ');

      sql = `UPDATE ${safeTable} SET ${setClause} WHERE id = ?`;
      params = updateKeys.map(k => wrap_param(record[k]));
      params.push(wrap_param(record.id));
      break;
    }

    case 'update_by_column':
      if (!id || !safeColumn) {
        return new Response('Missing id or column', { status: 400, headers: corsHeaders });
      }
      sql = `UPDATE ${safeTable} SET ${safeColumn} = ? WHERE id = ?`;
      params = [wrap_param(value), wrap_param(id)];
      break;

    case 'delete':
      if (!id) {
        return new Response('Missing id', { status: 400, headers: corsHeaders });
      }
      sql = `DELETE FROM ${safeTable} WHERE id = ?`;
      params = [wrap_param(id)];
      break;

    default:
      return new Response('Unknown action', { status: 400, headers: corsHeaders });
  }

  const missing = ['DB_URL', 'DB_TOKEN'].filter(name => !env[name]);
  if (missing.length > 0) {
    return new Response(`Missing DB credentials: ${missing.join(', ')}`, { status: 500, headers: corsHeaders });
  }

  const dbReq = {
    batch: {
      steps: [{ stmt: { sql, args: params } }]
    }
  };

  const resp = await fetch(`${env.DB_URL}/v1/batch`, {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${env.DB_TOKEN}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(dbReq)
  });

  if (!resp.ok) {
    const errText = await resp.text();
    return new Response(`DB error: ${errText}`, { status: 500, headers: corsHeaders });
  }

  let dbRes;
  try {
    dbRes = await resp.json();
  } catch {
    return new Response('Invalid DB response', { status: 500, headers: corsHeaders });
  }

  // Failed statements still come back as HTTP 200, with the error in step_errors
  const stepError = dbRes.result?.step_errors?.[0];
  if (stepError) {
    return new Response(`DB error: ${stepError.message}`, { status: 500, headers: corsHeaders });
  }

  const step = dbRes.result?.step_results?.[0];
  if (!step) {
    return new Response('Invalid DB response', { status: 500, headers: corsHeaders });
  }

  const json = (body) => new Response(JSON.stringify(body), { status: 200, headers: corsHeaders });

  switch (action) {
    case 'fetch':
      return json({ records: step.rows.map(row => row_to_object(step.cols, row)) });

    case 'fetch_by_id': {
      const row = step.rows[0];
      return json({ record: row ? row_to_object(step.cols, row) : null });
    }

    case 'fetch_by_user': {
      const row = step.rows[0];
      if (!row) {
        return new Response('Username does not exist', { status: 404, headers: corsHeaders });
      }
      return json({ id: cell_value(row[0]) });
    }

    case 'insert':
      return json({
        success: step.affected_row_count > 0,
        id: step.last_insert_rowid ? Number(step.last_insert_rowid) : 0
      });

    case 'update':
    case 'update_by_column':
      return json({ updated: step.affected_row_count });

    case 'delete':
      return json({ deleted: step.affected_row_count });

    default:
      return json({});
  }
}

function row_to_object(cols, row) {
  const obj = {};
  cols.forEach((col, i) => {
    obj[col.name] = cell_value(row[i]);
  });
  return obj;
}

function cell_value(cell) {
  if (!cell || cell.type === 'null') return null;
  if (cell.type === 'integer') return Number(cell.value);
  if (cell.type === 'blob') return cell.base64;
  return cell.value;
}

function safe_identifier(name) {
  if (!/^[a-zA-Z0-9_]+$/.test(name)) {
    throw new Error("Invalid identifier");
  }
  return name;
}

function wrap_param(val) {
  if (val === null || val === undefined) return { type: 'null' };

  if (typeof val === 'number') {
    if (Number.isInteger(val)) return { type: 'integer', value: String(val) };
    return { type: 'float', value: val };
  }

  if (typeof val === 'boolean') {
    return { type: 'integer', value: val ? '1' : '0' };
  }

  return { type: 'text', value: String(val) };
}
