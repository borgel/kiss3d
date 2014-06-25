// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/*jslint browser: true, es5: true */
/*globals $: true, rootPath: true */

(function() {
    "use strict";
    var resizeTimeout, interval;

    $('.js-only').removeClass('js-only');

    function getQueryStringParams() {
        var params = {};
        window.location.search.substring(1).split("&").
            map(function(s) {
                var pair = s.split("=");
                params[decodeURIComponent(pair[0])] =
                    typeof pair[1] === "undefined" ?
                            null : decodeURIComponent(pair[1]);
            });
        return params;
    }

    function browserSupportsHistoryApi() {
        return window.history && typeof window.history.pushState === "function";
    }

    function resizeShortBlocks() {
        if (resizeTimeout) {
            clearTimeout(resizeTimeout);
        }
        resizeTimeout = setTimeout(function() {
            var contentWidth = $('.content').width();
            $('.docblock.short').width(function() {
                return contentWidth - 40 - $(this).prev().width();
            }).addClass('nowrap');
        }, 150);
    }
    resizeShortBlocks();
    $(window).on('resize', resizeShortBlocks);

    function highlightSourceLines() {
        var i, from, to, match = window.location.hash.match(/^#?(\d+)(?:-(\d+))?$/);
        if (match) {
            from = parseInt(match[1], 10);
            to = Math.min(50000, parseInt(match[2] || match[1], 10));
            from = Math.min(from, to);
            if ($('#' + from).length === 0) {
                return;
            }
            $('#' + from)[0].scrollIntoView();
            $('.line-numbers span').removeClass('line-highlighted');
            for (i = from; i <= to; i += 1) {
                $('#' + i).addClass('line-highlighted');
            }
        }
    }
    highlightSourceLines();
    $(window).on('hashchange', highlightSourceLines);

    $(document).on('keyup', function(e) {
        if (document.activeElement.tagName === 'INPUT') {
            return;
        }

        if (e.which === 191 && $('#help').hasClass('hidden')) { // question mark
            e.preventDefault();
            $('#help').removeClass('hidden');
        } else if (e.which === 27) { // esc
            if (!$('#help').hasClass('hidden')) {
                e.preventDefault();
                $('#help').addClass('hidden');
            } else if (!$('#search').hasClass('hidden')) {
                e.preventDefault();
                $('#search').addClass('hidden');
                $('#main').removeClass('hidden');
            }
        } else if (e.which === 83) { // S
            e.preventDefault();
            $('.search-input').focus();
        }
    }).on('click', function(e) {
        if (!$(e.target).closest('#help').length) {
            $('#help').addClass('hidden');
        }
    });

    $('.version-selector').on('change', function() {
        var i, match,
            url = document.location.href,
            stripped = '',
            len = rootPath.match(/\.\.\//g).length + 1;

        for (i = 0; i < len; i += 1) {
            match = url.match(/\/[^\/]*$/);
            if (i < len - 1) {
                stripped = match[0] + stripped;
            }
            url = url.substring(0, url.length - match[0].length);
        }

        url += '/' + $('.version-selector').val() + stripped;

        document.location.href = url;
    });

    function initSearch(rawSearchIndex) {
        var currentResults, index, searchIndex;
        var params = getQueryStringParams();

        // Populate search bar with query string search term when provided,
        // but only if the input bar is empty. This avoid the obnoxious issue
        // where you start trying to do a search, and the index loads, and
        // suddenly your search is gone!
        if ($(".search-input")[0].value === "") {
            $(".search-input")[0].value = params.search || '';
        }

        /**
         * Executes the query and builds an index of results
         * @param  {[Object]} query     [The user query]
         * @param  {[type]} max         [The maximum results returned]
         * @param  {[type]} searchWords [The list of search words to query
         *                               against]
         * @return {[type]}             [A search index of results]
         */
        function execQuery(query, max, searchWords) {
            var valLower = query.query.toLowerCase(),
                val = valLower,
                typeFilter = itemTypeFromName(query.type),
                results = [],
                split = valLower.split("::");

            //remove empty keywords
            for (var j = 0; j < split.length; j++) {
                split[j].toLowerCase();
                if (split[j] === "") {
                    split.splice(j, 1);
                }
            }

            // quoted values mean literal search
            var nSearchWords = searchWords.length;
            if ((val.charAt(0) === "\"" || val.charAt(0) === "'") &&
                val.charAt(val.length - 1) === val.charAt(0))
            {
                val = val.substr(1, val.length - 2);
                for (var i = 0; i < nSearchWords; i += 1) {
                    if (searchWords[i] === val) {
                        // filter type: ... queries
                        if (typeFilter < 0 || typeFilter === searchIndex[i].ty) {
                            results.push({id: i, index: -1});
                        }
                    }
                    if (results.length === max) {
                        break;
                    }
                }
            } else {
                // gather matching search results up to a certain maximum
                val = val.replace(/\_/g, "");
                for (var i = 0; i < split.length; i++) {
                    for (var j = 0; j < nSearchWords; j += 1) {
                        if (searchWords[j].indexOf(split[i]) > -1 ||
                            searchWords[j].indexOf(val) > -1 ||
                            searchWords[j].replace(/_/g, "").indexOf(val) > -1)
                        {
                            // filter type: ... queries
                            if (typeFilter < 0 || typeFilter === searchIndex[j].ty) {
                                results.push({id: j, index: searchWords[j].replace(/_/g, "").indexOf(val)});
                            }
                        }
                        if (results.length === max) {
                            break;
                        }
                    }
                }
            }

            var nresults = results.length;
            for (var i = 0; i < nresults; i += 1) {
                results[i].word = searchWords[results[i].id];
                results[i].item = searchIndex[results[i].id] || {};
            }
            // if there are no results then return to default and fail
            if (results.length === 0) {
                return [];
            }

            results.sort(function(aaa, bbb) {
                var a, b;

                // sort by crate (non-current crate goes later)
                a = (aaa.item.crate !== window.currentCrate);
                b = (bbb.item.crate !== window.currentCrate);
                if (a !== b) return a - b;

                // sort by exact match (mismatch goes later)
                a = (aaa.word !== valLower);
                b = (bbb.word !== valLower);
                if (a !== b) return a - b;

                // sort by item name length (longer goes later)
                a = aaa.word.length;
                b = bbb.word.length;
                if (a !== b) return a - b;

                // sort by item name (lexicographically larger goes later)
                a = aaa.word;
                b = bbb.word;
                if (a !== b) return (a > b ? +1 : -1);

                // sort by index of keyword in item name (no literal occurrence goes later)
                a = (aaa.index < 0);
                b = (bbb.index < 0);
                if (a !== b) return a - b;
                // (later literal occurrence, if any, goes later)
                a = aaa.index;
                b = bbb.index;
                if (a !== b) return a - b;

                // sort by description (no description goes later)
                a = (aaa.item.desc === '');
                b = (bbb.item.desc === '');
                if (a !== b) return a - b;

                // sort by type (later occurrence in `itemTypes` goes later)
                a = aaa.item.ty;
                b = bbb.item.ty;
                if (a !== b) return a - b;

                // sort by path (lexicographically larger goes later)
                a = aaa.item.path;
                b = bbb.item.path;
                if (a !== b) return (a > b ? +1 : -1);

                // que sera, sera
                return 0;
            });

            // remove duplicates, according to the data provided
            for (var i = results.length - 1; i > 0; i -= 1) {
                if (results[i].word === results[i - 1].word &&
                    results[i].item.ty === results[i - 1].item.ty &&
                    results[i].item.path === results[i - 1].item.path)
                {
                    results[i].id = -1;
                }
            }
            for (var i = 0; i < results.length; i++) {
                var result = results[i],
                    name = result.item.name.toLowerCase(),
                    path = result.item.path.toLowerCase(),
                    parent = result.item.parent;

                var valid = validateResult(name, path, split, parent);
                if (!valid) {
                    result.id = -1;
                }
            }
            return results;
        }

        /**
         * Validate performs the following boolean logic. For example:
         * "File::open" will give IF A PARENT EXISTS => ("file" && "open")
         * exists in (name || path || parent) OR => ("file" && "open") exists in
         * (name || path )
         *
         * This could be written functionally, but I wanted to minimise
         * functions on stack.
         *
         * @param  {[string]} name   [The name of the result]
         * @param  {[string]} path   [The path of the result]
         * @param  {[string]} keys   [The keys to be used (["file", "open"])]
         * @param  {[object]} parent [The parent of the result]
         * @return {[boolean]}       [Whether the result is valid or not]
         */
        function validateResult(name, path, keys, parent) {
            //initially valid
            var validate = true;
            //if there is a parent, then validate against parent
            if (parent !== undefined) {
                for (var i = 0; i < keys.length; i++) {
                    // if previous keys are valid and current key is in the
                    // path, name or parent
                    if ((validate) &&
                        (name.toLowerCase().indexOf(keys[i]) > -1 ||
                         path.toLowerCase().indexOf(keys[i]) > -1 ||
                         parent.name.toLowerCase().indexOf(keys[i]) > -1))
                    {
                        validate = true;
                    } else {
                        validate = false;
                    }
                }
            } else {
                for (var i = 0; i < keys.length; i++) {
                    // if previous keys are valid and current key is in the
                    // path, name
                    if ((validate) &&
                        (name.toLowerCase().indexOf(keys[i]) > -1 ||
                         path.toLowerCase().indexOf(keys[i]) > -1))
                    {
                        validate = true;
                    } else {
                        validate = false;
                    }
                }
            }
            return validate;
        }

        function getQuery() {
            var matches, type, query = $('.search-input').val();

            matches = query.match(/^(fn|mod|str(uct)?|enum|trait|t(ype)?d(ef)?)\s*:\s*/i);
            if (matches) {
                type = matches[1].replace(/^td$/, 'typedef')
                                 .replace(/^str$/, 'struct')
                                 .replace(/^tdef$/, 'typedef')
                                 .replace(/^typed$/, 'typedef');
                query = query.substring(matches[0].length);
            }

            return {
                query: query,
                type: type,
                id: query + type,
            };
        }

        function initSearchNav() {
            var hoverTimeout, $results = $('.search-results .result');

            $results.on('click', function() {
                var dst = $(this).find('a')[0];
                if (window.location.pathname == dst.pathname) {
                    $('#search').addClass('hidden');
                    $('#main').removeClass('hidden');
                }
                document.location.href = dst.href;
            }).on('mouseover', function() {
                var $el = $(this);
                clearTimeout(hoverTimeout);
                hoverTimeout = setTimeout(function() {
                    $results.removeClass('highlighted');
                    $el.addClass('highlighted');
                }, 20);
            });

            $(document).off('keypress.searchnav');
            $(document).on('keypress.searchnav', function(e) {
                var $active = $results.filter('.highlighted');

                if (e.which === 38) { // up
                    e.preventDefault();
                    if (!$active.length || !$active.prev()) {
                        return;
                    }

                    $active.prev().addClass('highlighted');
                    $active.removeClass('highlighted');
                } else if (e.which === 40) { // down
                    e.preventDefault();
                    if (!$active.length) {
                        $results.first().addClass('highlighted');
                    } else if ($active.next().length) {
                        $active.next().addClass('highlighted');
                        $active.removeClass('highlighted');
                    }
                } else if (e.which === 13) { // return
                    e.preventDefault();
                    if ($active.length) {
                        document.location.href = $active.find('a').prop('href');
                    }
                }
            });
        }

        function escape(content) {
            return $('<h1/>').text(content).html();
        }

        function showResults(results) {
            var output, shown, query = getQuery();

            currentResults = query.id;
            output = '<h1>Results for ' + escape(query.query) +
                (query.type ? ' (type: ' + escape(query.type) + ')' : '') + '</h1>';
            output += '<table class="search-results">';

            if (results.length > 0) {
                shown = [];

                results.forEach(function(item) {
                    var name, type;

                    if (shown.indexOf(item) !== -1) {
                        return;
                    }

                    shown.push(item);
                    name = item.name;
                    type = itemTypes[item.ty];

                    output += '<tr class="' + type + ' result"><td>';

                    if (type === 'mod') {
                        output += item.path +
                            '::<a href="' + rootPath +
                            item.path.replace(/::/g, '/') + '/' +
                            name + '/index.html" class="' +
                            type + '">' + name + '</a>';
                    } else if (type === 'static' || type === 'reexport') {
                        output += item.path +
                            '::<a href="' + rootPath +
                            item.path.replace(/::/g, '/') +
                            '/index.html" class="' + type +
                            '">' + name + '</a>';
                    } else if (item.parent !== undefined) {
                        var myparent = item.parent;
                        var anchor = '#' + type + '.' + name;
                        output += item.path + '::' + myparent.name +
                            '::<a href="' + rootPath +
                            item.path.replace(/::/g, '/') +
                            '/' + itemTypes[myparent.ty] +
                            '.' + myparent.name +
                            '.html' + anchor +
                            '" class="' + type +
                            '">' + name + '</a>';
                    } else {
                        output += item.path +
                            '::<a href="' + rootPath +
                            item.path.replace(/::/g, '/') +
                            '/' + type +
                            '.' + name +
                            '.html" class="' + type +
                            '">' + name + '</a>';
                    }

                    output += '</td><td><span class="desc">' + item.desc +
                        '</span></td></tr>';
                });
            } else {
                output += 'No results :( <a href="https://duckduckgo.com/?q=' +
                    encodeURIComponent('rust ' + query.query) +
                    '">Try on DuckDuckGo?</a>';
            }

            output += "</p>";
            $('#main.content').addClass('hidden');
            $('#search.content').removeClass('hidden').html(output);
            $('#search .desc').width($('#search').width() - 40 -
                $('#search td:first-child').first().width());
            initSearchNav();
        }

        function search(e) {
            var query,
                filterdata = [],
                obj, i, len,
                results = [],
                maxResults = 200,
                resultIndex;
            var params = getQueryStringParams();

            query = getQuery();
            if (e) {
                e.preventDefault();
            }

            if (!query.query || query.id === currentResults) {
                return;
            }

            // Because searching is incremental by character, only the most
            // recent search query is added to the browser history.
            if (browserSupportsHistoryApi()) {
                if (!history.state && !params.search) {
                    history.pushState(query, "", "?search=" +
                                                encodeURIComponent(query.query));
                } else {
                    history.replaceState(query, "", "?search=" +
                                                encodeURIComponent(query.query));
                }
            }

            resultIndex = execQuery(query, 20000, index);
            len = resultIndex.length;
            for (i = 0; i < len; i += 1) {
                if (resultIndex[i].id > -1) {
                    obj = searchIndex[resultIndex[i].id];
                    filterdata.push([obj.name, obj.ty, obj.path, obj.desc]);
                    results.push(obj);
                }
                if (results.length >= maxResults) {
                    break;
                }
            }

            showResults(results);
        }

        // This mapping table should match the discriminants of
        // `rustdoc::html::item_type::ItemType` type in Rust.
        var itemTypes = ["mod",
                         "struct",
                         "type",
                         "fn",
                         "type",
                         "static",
                         "trait",
                         "impl",
                         "viewitem",
                         "tymethod",
                         "method",
                         "structfield",
                         "variant",
                         "ffi",
                         "ffs",
                         "macro",
                         "primitive"];

        function itemTypeFromName(typename) {
            for (var i = 0; i < itemTypes.length; ++i) {
                if (itemTypes[i] === typename) return i;
            }
            return -1;
        }

        function buildIndex(rawSearchIndex) {
            searchIndex = [];
            var searchWords = [];
            for (var crate in rawSearchIndex) {
                if (!rawSearchIndex.hasOwnProperty(crate)) { continue }

                // an array of [(Number) item type,
                //              (String) name,
                //              (String) full path or empty string for previous path,
                //              (String) description,
                //              (optional Number) the parent path index to `paths`]
                var items = rawSearchIndex[crate].items;
                // an array of [(Number) item type,
                //              (String) name]
                var paths = rawSearchIndex[crate].paths;

                // convert `paths` into an object form
                var len = paths.length;
                for (var i = 0; i < len; ++i) {
                    paths[i] = {ty: paths[i][0], name: paths[i][1]};
                }

                // convert `items` into an object form, and construct word indices.
                //
                // before any analysis is performed lets gather the search terms to
                // search against apart from the rest of the data.  This is a quick
                // operation that is cached for the life of the page state so that
                // all other search operations have access to this cached data for
                // faster analysis operations
                var len = items.length;
                var lastPath = "";
                for (var i = 0; i < len; i += 1) {
                    var rawRow = items[i];
                    var row = {crate: crate, ty: rawRow[0], name: rawRow[1],
                               path: rawRow[2] || lastPath, desc: rawRow[3],
                               parent: paths[rawRow[4]]};
                    searchIndex.push(row);
                    if (typeof row.name === "string") {
                        var word = row.name.toLowerCase();
                        searchWords.push(word);
                    } else {
                        searchWords.push("");
                    }
                    lastPath = row.path;
                }
            }
            return searchWords;
        }

        function startSearch() {
            var keyUpTimeout;
            $('.do-search').on('click', search);
            $('.search-input').on('keyup', function() {
                clearTimeout(keyUpTimeout);
                keyUpTimeout = setTimeout(search, 100);
            });

            // Push and pop states are used to add search results to the browser
            // history.
            if (browserSupportsHistoryApi()) {
                $(window).on('popstate', function(e) {
                    var params = getQueryStringParams();
                    // When browsing back from search results the main page
                    // visibility must be reset.
                    if (!params.search) {
                        $('#main.content').removeClass('hidden');
                        $('#search.content').addClass('hidden');
                    }
                    // When browsing forward to search results the previous
                    // search will be repeated, so the currentResults are
                    // cleared to ensure the search is successful.
                    currentResults = null;
                    // Synchronize search bar with query string state and
                    // perform the search. This will empty the bar if there's
                    // nothing there, which lets you really go back to a
                    // previous state with nothing in the bar.
                    $('.search-input').val(params.search);
                    // Some browsers fire 'onpopstate' for every page load
                    // (Chrome), while others fire the event only when actually
                    // popping a state (Firefox), which is why search() is
                    // called both here and at the end of the startSearch()
                    // function.
                    search();
                });
            }
            search();
        }

        index = buildIndex(rawSearchIndex);
        startSearch();

        // Draw a convenient sidebar of known crates if we have a listing
        if (rootPath == '../') {
            var sidebar = $('.sidebar');
            var div = $('<div>').attr('class', 'block crate');
            div.append($('<h2>').text('Crates'));

            var crates = [];
            for (var crate in rawSearchIndex) {
                if (!rawSearchIndex.hasOwnProperty(crate)) { continue }
                crates.push(crate);
            }
            crates.sort();
            for (var i = 0; i < crates.length; i++) {
                var klass = 'crate';
                if (crates[i] == window.currentCrate) {
                    klass += ' current';
                }
                div.append($('<a>', {'href': '../' + crates[i] + '/index.html',
                                    'class': klass}).text(crates[i]));
            }
            sidebar.append(div);
        }
    }

    window.initSearch = initSearch;

    window.register_implementors = function(imp) {
        var list = $('#implementors-list');
        var libs = Object.getOwnPropertyNames(imp);
        for (var i = 0; i < libs.length; i++) {
            if (libs[i] == currentCrate) continue;
            var structs = imp[libs[i]];
            for (var j = 0; j < structs.length; j++) {
                var code = $('<code>').append(structs[j]);
                $.each(code.find('a'), function(idx, a) {
                    var href = $(a).attr('href');
                    if (!href.startsWith('http')) {
                        $(a).attr('href', rootPath + $(a).attr('href'));
                    }
                });
                var li = $('<li>').append(code);
                list.append(li);
            }
        }
    };
    if (window.pending_implementors) {
        window.register_implementors(window.pending_implementors);
    }

    // See documentation in html/render.rs for what this is doing.
    var query = getQueryStringParams();
    if (query['gotosrc']) {
        window.location = $('#src-' + query['gotosrc']).attr('href');
    }
}());