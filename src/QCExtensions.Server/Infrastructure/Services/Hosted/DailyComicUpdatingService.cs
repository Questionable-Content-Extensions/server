using System;
using System.Net.Http;
using System.Threading;
using System.Threading.Tasks;
using AngleSharp.Html.Dom;
using AngleSharp.Html.Parser;
using Microsoft.AspNetCore.Hosting;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;
using QCExtensions.Application.Extensions.DbContext;
using QCExtensions.Application.Interfaces;
using QCExtensions.Domain.Entities;

namespace QCExtensions.Server.Infrastructure.Services.Hosted
{
	public class DailyComicUpdatingService : RepeatingBackgroundService
	{
		private static HttpClient m_httpClient = new HttpClient();

		private const string QCFrontPageUrl = "https://questionablecontent.net/";
		private const string QCArchiveUrl = QCFrontPageUrl + "archive.php";
		private readonly TimeZoneInfo QCTimeZone = TimeZoneInfo.FindSystemTimeZoneById(Environment.GetEnvironmentVariable("QC_TIMEZONE"));

		private readonly IDateTime _dateTime;
		private IServiceProvider _provider;
		private ILogger<DailyComicUpdatingService> _logger;
		private readonly IHostingEnvironment _env;

		public DailyComicUpdatingService(
			IDateTime dateTime,
			IServiceProvider serviceProvider,
			ILogger<DailyComicUpdatingService> logger,
			IHostingEnvironment env)
		{
			_dateTime = dateTime;
			_provider = serviceProvider;
			_logger = logger;
			_env = env;
		}

		protected override async Task<bool> ExecuteRepeatedlyAsync(CancellationToken stoppingToken)
		{
			var now = _dateTime.Now;
			using (var scope = _provider.CreateScope())
			{
				var context = scope.ServiceProvider.GetRequiredService<DomainDbContext>();
				_logger.LogInformation($"We do not yet have the data for the comic on {now.Date:dddd, dd MMMM yyyy}. Fetching QC front page.");
				try
				{
					await Task.Delay(TimeSpan.FromSeconds(15));
					if (_env.IsProduction())
					{
						await FetchLatestComicDataAsync(context);
					}
					else
					{
						_logger.LogInformation($"Would do {nameof(FetchLatestComicDataAsync)} now if running in production environment");
					}
				}
				catch (Exception e)
				{
					_logger.LogError(e, "Could not fetch latest comic data, exception occurred");
				}
			}

			TimeSpan delay = GetTimeUntilNextUpdate(now);
			_logger.LogInformation($"Waiting for {delay} until next update.");
			await Task.Delay(delay, stoppingToken);

			return true;
		}

		private static TimeSpan GetTimeUntilNextUpdate(DateTime now)
		{
			// Check the day of week. If Saturday or Sunday, only check once (at noon)
			// Otherwise, check at these hours: 0-3,6,12,18,21-23.
			TimeSpan delay;
			if (now.DayOfWeek == DayOfWeek.Saturday)
			{
				if (now.TimeOfDay < TimeSpan.FromHours(12))
				{
					delay = TimeSpan.FromHours(12) - now.TimeOfDay;
				}
				else
				{
					delay = TimeSpan.FromHours(24) - (now.TimeOfDay - TimeSpan.FromHours(12));
				}
			}
			else if (now.DayOfWeek == DayOfWeek.Sunday)
			{
				if (now.TimeOfDay < TimeSpan.FromHours(12))
				{
					delay = TimeSpan.FromHours(12) - now.TimeOfDay;
				}
				else
				{
					delay = TimeSpan.FromHours(24) - now.TimeOfDay;
				}
			}
			else
			{
				if (now.TimeOfDay.Hours < 3)
				{
					delay = TimeSpan.FromHours(1) - (now.TimeOfDay - TimeSpan.FromHours(now.TimeOfDay.Hours));
				}
				else if (now.TimeOfDay.Hours < 6)
				{
					delay = TimeSpan.FromHours(6) - now.TimeOfDay;
				}
				else if (now.TimeOfDay.Hours < 12)
				{
					delay = TimeSpan.FromHours(12) - now.TimeOfDay;
				}
				else if (now.TimeOfDay.Hours < 18)
				{
					delay = TimeSpan.FromHours(18) - now.TimeOfDay;
				}
				else if (now.TimeOfDay.Hours < 21)
				{
					delay = TimeSpan.FromHours(21) - now.TimeOfDay;
				}
				else
				{
					delay = TimeSpan.FromHours(1) - (now.TimeOfDay - TimeSpan.FromHours(now.TimeOfDay.Hours));
				}
			}

			return delay;
		}

		private async Task<IHtmlDocument> GetHtmlDocumentAsync(string url, string what)
		{
			var response = await m_httpClient.GetAsync(url);
			if (!response.IsSuccessStatusCode)
			{
				_logger.LogWarning($"Could not fetch {what}, got HTTP status {response.StatusCode}");
				return null;
			}
			var content = await response.Content.ReadAsStringAsync();
			if (string.IsNullOrEmpty(content))
			{
				_logger.LogWarning($"Could not fetch {what}, got empty response");
				return null;
			}

			var parser = new HtmlParser();
			return parser.ParseDocument(content);
		}

		private async Task FetchLatestComicDataAsync(DomainDbContext context)
		{
			var qcFrontPageDocument = await GetHtmlDocumentAsync(QCFrontPageUrl, "latest comic data");
			if (qcFrontPageDocument == null) return;

			var comicImageElement = qcFrontPageDocument.QuerySelector("img[src*=\"/comics/\"]") as IHtmlImageElement;
			var comicImageUrl = comicImageElement.Source;
			var comicId = int.Parse(comicImageUrl.Substring(comicImageUrl.LastIndexOf('/') + 1, comicImageUrl.LastIndexOf('.') - comicImageUrl.LastIndexOf('/') - 1));

			using (var transaction = context.Database.BeginTransaction())
			{
				var (comic, wasCreated) = await context.Comics.GetOrCreateAsync(comicId);
				if (string.IsNullOrEmpty(comic.Title))
				{
					var qcArchiveDocument = await GetHtmlDocumentAsync(QCArchiveUrl, "archive data");
					if (qcArchiveDocument != null)
					{
						var comicTitleElement = qcArchiveDocument.QuerySelector($"a[href*=\"comic={comicId}\"]");
						var comicTitle = comicTitleElement.InnerHtml.Split(':', 2)[1].Trim();
						comic.Title = comicTitle;
					}
				}
				if (wasCreated) { await context.SaveChangesAsync(); }

				var newsElement = qcFrontPageDocument.GetElementById("newspost");
				var dateElement = newsElement?.PreviousElementSibling?.QuerySelector("b");
				if (dateElement == null)
				{
					_logger.LogWarning($"Could not fetch date for latest comic #{comicId}, couldn't find date element on page");
				}
				else
				{
					var date = DateTime.Parse(dateElement.InnerHtml);
					var utcDate = TimeZoneInfo.ConvertTimeToUtc(date, QCTimeZone);
					comic.PublishDate = utcDate;
					comic.IsAccuratePublishDate = true;
				}

				// Grab the news
				if (newsElement == null)
				{
					_logger.LogWarning($"Could not fetch news for latest comic #{comicId}, couldn't find news element on page");
				}
				else
				{
					foreach (var unwantedElements in newsElement.QuerySelectorAll("script, hr"))
					{
						unwantedElements.Remove();
					}

					var newsText = newsElement.InnerHtml
						.Replace("\r", "")
						.Replace("\n", "")
						.Replace("<br />", "<br>")
						.Replace("<br/>", "<br>")
						.Replace("<br>", "\n")
						.Trim();

					var newsEntity = await context.News.SingleOrDefaultAsync(n => n.ComicId == comicId);
					if (newsEntity == null)
					{
						// New news
						newsEntity = new News
						{
							ComicId = comicId,
							LastUpdated = DateTime.UtcNow,
							UpdateFactor = 1,
							NewsText = newsText
						};
						context.News.Add(newsEntity);
					}
					else
					{
						// Old news. Compare news text with old.
						if (newsEntity.NewsText == newsText)
						{
							_logger.LogInformation($"News text for latest comic #{comicId} hasn't changed.");
						}
						else
						{
							_logger.LogInformation($"News text for comic #{comicId} has changed. Resetting update factor and updating text.");
							newsEntity.UpdateFactor = 1;
							newsEntity.NewsText = newsText;
							newsEntity.LastUpdated = DateTime.UtcNow;
							context.News.Update(newsEntity);
						}
					}
				}

				_logger.LogInformation($"Saving any changes to the database.");
				await context.SaveChangesAsync();

				transaction.Commit();
			}
		}
	}
}